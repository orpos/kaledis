//! Portable executable image representation.
//!
//! See <https://learn.microsoft.com/en-us/windows/win32/debug/pe-format> for more information.

use std::borrow::Cow;

use ahash::RandomState;
use indexmap::IndexMap;
use log::{debug, error, info, warn};
use zerocopy::IntoBytes;

use super::{constants::*, errors::*, resource::*, types::*, util::*};

/// Image data directory type enumeration.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DataDirectoryType {
    ExportTable,
    ImportTable,
    ResourceTable,
    ExceptionTable,
    CertificateTable,
    BaseRelocationTable,
    Debug,
    Architecture,
    GlobalPtr,
    TLSTable,
    LoadConfigTable,
    BoundImport,
    IAT,
    DelayImportDescriptor,
    CLRRuntimeHeader,
    Reserved,
}

/// Portable executable image representation.
///
/// This struct is the main entry point for parsing, querying and updating a portable executable image.
#[derive(Debug, Clone)]
pub struct Image<'a> {
    pub(crate) image: Cow<'a, [u8]>,

    pub(crate) pe_dos_magic: u16,
    pub(crate) pe_signature: u32,
    pub(crate) coff_header: CoffHeader,
    pub(crate) standard_header: StandardHeader,
    pub(crate) windows_header: GenericWindowsHeader,
    pub(crate) header_data_directory: IndexMap<DataDirectoryType, ImageDataDirectory, RandomState>,
    pub(crate) section_table: Vec<SectionHeader>,

    pub(crate) resource_directory: Option<ResourceDirectory>,

    coff_header_offset: u64,
    optional_header_dd_offset: u64,
    directories_offset: u64,
}

impl PartialEq for Image<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.pe_dos_magic == other.pe_dos_magic
            && self.pe_signature == other.pe_signature
            && self.coff_header == other.coff_header
            && self.standard_header == other.standard_header
            && self.windows_header == other.windows_header
            && self.header_data_directory == other.header_data_directory
            && self.section_table == other.section_table
            && self.resource_directory == other.resource_directory
    }
}
impl Eq for Image<'_> {}

impl<'a> Image<'a> {
    /// Parse a portable executable image from a byte slice.
    ///
    /// # Returns
    /// Returns the `Image`, or an error if the byte slice is not a valid portable executable image or is missing required headers.
    pub fn parse<R: Into<Cow<'a, [u8]>>>(image: R) -> Result<Self, ImageReadError> {
        let image = image.into();

        let pe_dos_magic = read::<u16>(&image[0..])?;
        debug!("pe_dos_magic: {:#x?}", pe_dos_magic);
        if pe_dos_magic != PE_DOS_MAGIC {
            return Err(ImageReadError::InvalidHeader("no dos magic".into()));
        }

        let pe_signature_offset = read::<u32>(&image[PE_PTR_OFFSET as usize..])?;
        debug!("pe_signature_offset: {:#x?}", pe_signature_offset);

        let pe_signature = read::<u32>(&image[pe_signature_offset as usize..])?;
        debug!("pe_signature: {:#x?}", pe_signature);
        if pe_signature != PE_NT_SIGNATURE {
            return Err(ImageReadError::InvalidHeader("no pe signature".into()));
        }

        let coff_header_offset = (pe_signature_offset + 4) as u64;
        let coff_header = read::<CoffHeader>(&image[coff_header_offset as usize..])?;
        debug!("{:#x?}: {:#x?}", coff_header_offset, coff_header);
        if coff_header.size_of_optional_header < 24 {
            return Err(ImageReadError::InvalidHeader(
                "optional header too small".into(),
            ));
        }

        let standard_header_offset = coff_header_offset + 20;
        let standard_header = read::<StandardHeader>(&image[standard_header_offset as usize..])?;
        debug!("{:#x?}: {:#x?}", standard_header_offset, standard_header);

        let (
            windows_header_offset,
            windows_header,
            number_of_rva_and_sizes,
            optional_header_dd_offset,
        ) = {
            if standard_header.magic == PE_32_MAGIC && coff_header.size_of_optional_header >= 96 {
                let windows_header_offset = standard_header_offset + 28;
                let windows_header =
                    read::<WindowsHeader<u32>>(&image[windows_header_offset as usize..])?;
                (
                    windows_header_offset,
                    GenericWindowsHeader::WindowsHeader32(windows_header),
                    windows_header.number_of_rva_and_sizes,
                    standard_header_offset + 96,
                )
            } else if standard_header.magic == PE_64_MAGIC
                && coff_header.size_of_optional_header >= 112
            {
                let windows_header_offset = standard_header_offset + 24;
                let windows_header =
                    read::<WindowsHeader<u64>>(&image[windows_header_offset as usize..])?;
                (
                    windows_header_offset,
                    GenericWindowsHeader::WindowsHeader64(windows_header),
                    windows_header.number_of_rva_and_sizes,
                    standard_header_offset + 112,
                )
            } else {
                return Err(ImageReadError::InvalidHeader(
                    "invalid optional header".into(),
                ));
            }
        };
        debug!("{:#x?}: {:#x?}", windows_header_offset, windows_header);

        if image.len() <= optional_header_dd_offset as usize {
            return Err(ImageReadError::InvalidHeader(
                "image truncated after optional header".into(),
            ));
        }

        debug!(
            "optional_header_dd_offset: {:#x?}",
            optional_header_dd_offset,
        );
        let mut header_data_directory =
            IndexMap::<DataDirectoryType, ImageDataDirectory, _>::with_hasher(RandomState::new());
        use DataDirectoryType::*;
        for (index, &header) in [
            ExportTable,
            ImportTable,
            ResourceTable,
            ExceptionTable,
            CertificateTable,
            BaseRelocationTable,
            Debug,
            Architecture,
            GlobalPtr,
            TLSTable,
            LoadConfigTable,
            BoundImport,
            IAT,
            DelayImportDescriptor,
            CLRRuntimeHeader,
            Reserved,
        ]
        .iter()
        .enumerate()
        {
            if (index as u32) < number_of_rva_and_sizes {
                let offset = optional_header_dd_offset + (index * 8) as u64;
                let data = read::<ImageDataDirectory>(&image[offset as usize..])?;
                header_data_directory.insert(header, data);
                debug!("{:#x?}: {:?}: {:#x?}", offset, header, data);
            }
        }

        let section_table_offset =
            standard_header_offset + coff_header.size_of_optional_header as u64;
        let mut section_table = Vec::new();
        for index in 0..coff_header.number_of_sections {
            let section_table_offset = section_table_offset + (index * 40) as u64;
            let section_header = read::<SectionHeader>(&image[section_table_offset as usize..])?;
            debug!(
                "{:#x?}: {}: {:#x?}",
                section_table_offset,
                section_header.name().unwrap_or("?".to_string()),
                section_header
            );
            section_table.push(section_header);
        }

        let directories_offset =
            section_table_offset + (coff_header.number_of_sections * 40) as u64;

        let mut resource_directory = None;
        if let Some(resource_data) = header_data_directory.get(&DataDirectoryType::ResourceTable) {
            if resource_data.virtual_address > 0 && resource_data.size > 0 {
                for section_table in section_table.iter() {
                    if resource_data.virtual_address >= section_table.virtual_address
                        && resource_data.virtual_address
                            < section_table.virtual_address + section_table.virtual_size
                    {
                        debug!(
                            "found resource directory in {} section: {:#x?}",
                            section_table.name().unwrap_or("?".to_string()),
                            section_table
                        );
                        resource_directory = Some(ResourceDirectory::parse(
                            &image,
                            section_table.pointer_to_raw_data,
                            section_table.virtual_address,
                        )?);
                    }
                }
            }
        }

        Ok(Self {
            image,
            pe_dos_magic,
            pe_signature,
            coff_header,
            standard_header,
            windows_header,
            header_data_directory,
            section_table,
            resource_directory,
            coff_header_offset,
            optional_header_dd_offset,
            directories_offset,
        })
    }

    /// Parse a portable executable image from a file.
    ///
    /// # Returns
    /// Returns the `Image`, or an error if the file could not be read, is not a valid portable executable image or is missing required headers.
    pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ImageReadError> {
        let data = std::fs::read(path)?;
        Self::parse(data)
    }

    /// Parse a portable executable image from a reader.
    ///
    /// # Returns
    /// Returns the `Image`, or an error if the reader could not be read, is not a valid portable executable image or is missing required headers.
    pub fn parse_reader<R: std::io::Read>(reader: &mut R) -> Result<Self, ImageReadError> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::parse(data)
    }

    /// Write the portable executable image to a file.
    ///
    /// # Returns
    /// Returns an error if the file could not be written.
    pub fn write_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ImageWriteError> {
        std::fs::write(path, &self.image).map_err(|e| e.into())
    }

    /// Write the portable executable image to a writer.
    ///
    /// # Returns
    /// Returns an error if the writer could not be written.
    pub fn write_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<(), ImageWriteError> {
        writer.write_all(&self.image).map_err(|e| e.into())
    }

    /// Set the resource directory of the image.
    ///
    /// This will update the resource data directory and the resource section.
    /// If a section containing a resource directory already exists, it will be updated in place if the following conditions are met:
    /// - The new directory is not larger than the section containing the existing one, or that section is the last section in the image.
    /// - The section is not used by other directories.
    ///
    /// Otherwise, the existing section will be kept intact and a new section will be added after all other sections and before any other data at the end of the image.
    ///
    /// # Returns
    /// Returns the previous resource directory, or an error in the following cases:
    /// - Returns an error if the image could not be built. This can happen if there is not enough space in the image header to add a new section.
    /// - Returns an error if a section points to data outside the image.
    ///
    /// # Safety
    /// Replacing an existing resource directory may cause the resulting image to be invalid.
    /// Applications might reference data inside the resource directory that may not exist in the new one.
    /// Only set a resource directory originating from the same image with required resources intact unless you know what you are doing.
    ///
    /// Some packed images (e.g. packed with UPX) might not work with a modified resource directory or additional sections.
    pub fn set_resource_directory(
        &mut self,
        resource_directory: ResourceDirectory,
    ) -> Result<Option<ResourceDirectory>, ImageWriteError> {
        // copy to-be-modified data to allow erroring out without invalidating the image
        let mut coff_header = self.coff_header;
        let mut windows_header = self.windows_header;
        let mut header_data_directory = self.header_data_directory.clone();
        let mut section_table = self.section_table.clone();

        let mut required_header_space = 0;

        // ensure that the data directory entry for the resource table exists
        use DataDirectoryType::*;
        for (index, &header) in [ExportTable, ImportTable, ResourceTable].iter().enumerate() {
            if index + 1 > header_data_directory.len() {
                debug!("adding missing header data directory: {:?}", header);
                header_data_directory.insert(header, ImageDataDirectory::default());
                required_header_space += 8;
            }
        }
        let old_resource_data_directory = *header_data_directory
            .get(&DataDirectoryType::ResourceTable)
            .unwrap();

        let new_resource_directory_size = resource_directory.size();
        let new_resource_directory_size_aligned =
            aligned_to(resource_directory.size(), windows_header.file_alignment());
        debug!(
            "new resource data size: {:#x?} (aligned: {:#x?})",
            new_resource_directory_size, new_resource_directory_size_aligned
        );
        let mut resource_section_data = Vec::new();
        let mut new_section_data = Vec::new();

        let mut new_image = Vec::with_capacity(self.image.len());
        new_image.extend_from_slice(&self.image[..self.coff_header_offset as usize]);

        let first_section = section_table
            .iter()
            .filter(|section_header| section_header.size_of_raw_data > 0)
            .min_by_key(|section_header| section_header.pointer_to_raw_data)
            .copied();
        let first_section_start = first_section
            .map(|section| section.pointer_to_raw_data as usize)
            .unwrap_or(self.image.len());

        let last_section = section_table
            .iter()
            .filter(|section_header| section_header.size_of_raw_data > 0)
            .max_by_key(|section_header| {
                section_header.pointer_to_raw_data + section_header.size_of_raw_data
            })
            .copied();
        let last_section_end = last_section
            .map(|section| section.pointer_to_raw_data as usize + section.size_of_raw_data as usize)
            .unwrap_or(self.image.len());

        if last_section_end > self.image.len() {
            return Err(ImageWriteError::InvalidSectionRange(
                last_section_end as u64,
                self.image.len() as u64,
            ));
        }

        let mut old_resource_section_start = 0;
        let mut old_resource_section_end = 0;
        let mut old_resource_section = None;
        if old_resource_data_directory.size > 0 {
            debug!(
                "resource directory exists in the data directory table (size: {:#x?})",
                old_resource_data_directory.size
            );
            // search for the section containing the resource directory
            for section_header in section_table.iter_mut() {
                if old_resource_data_directory.virtual_address >= section_header.virtual_address
                    && old_resource_data_directory.virtual_address
                        < section_header.virtual_address + section_header.virtual_size
                {
                    debug!(
                        "found existing resource directory in {} section: {:#x?}",
                        section_header.name().unwrap_or("?".to_string()),
                        section_header
                    );
                    old_resource_section_start = section_header.pointer_to_raw_data as usize;
                    old_resource_section_end =
                        old_resource_section_start + section_header.size_of_raw_data as usize;
                    old_resource_section = Some(section_header);
                    break;
                }
            }
        }

        let mut add_new_section = true;
        let mut multiple_data_directories = false;
        if let Some(ref mut old_resource_section) = old_resource_section {
            // an existing resource section was found
            let last_section = last_section.unwrap();
            let is_last_section = last_section.pointer_to_raw_data + last_section.size_of_raw_data
                == old_resource_section.pointer_to_raw_data + old_resource_section.size_of_raw_data;

            // check if the existing section is large enough to hold the new resource directory
            // or if it is the last section and can be extended
            if old_resource_section.size_of_raw_data >= new_resource_directory_size {
                debug!(
                    "existing section size is large enough and can be reused ({:#x?} >= {:#x?})",
                    old_resource_section.size_of_raw_data, new_resource_directory_size
                );
                add_new_section = false;
            } else if is_last_section {
                debug!(
                    "existing section is the last section and can be extended ({:#x?} < {:#x?})",
                    old_resource_section.size_of_raw_data, new_resource_directory_size
                );
                add_new_section = false;
            } else {
                debug!(
                    "existing resource section size is too small and followed by other sections ({:#x?} < {:#x?})",
                    old_resource_section.size_of_raw_data, new_resource_directory_size
                );
            }

            if !add_new_section {
                // check for other sections also using the resource section
                for (header, directory) in header_data_directory.iter() {
                    if header != &ResourceTable
                        && directory.virtual_address >= old_resource_section.virtual_address
                        && directory.virtual_address
                            < old_resource_section.virtual_address
                                + old_resource_section.virtual_size
                    {
                        info!("resource section also used by data directory {:?}", header);
                        multiple_data_directories = true;
                    }
                }

                if !multiple_data_directories {
                    // only the resource directory uses the resource section, we can replace and extend it
                    debug!("resource section only used by the resource table, overwriting section");
                    let resource_dd = header_data_directory
                        .get_mut(&DataDirectoryType::ResourceTable)
                        .unwrap();

                    if old_resource_data_directory.size >= new_resource_directory_size {
                        resource_section_data =
                            resource_directory.build(old_resource_data_directory.virtual_address);

                        if !is_last_section
                            && old_resource_section.size_of_raw_data > new_resource_directory_size
                        {
                            debug!(
                                "resource section is not the last section and smaller than the existing section, padding section with existing data"
                            );
                            // pad the section to the previous section size with existing data
                            resource_section_data.extend(
                                &self.image[(old_resource_section.pointer_to_raw_data as usize
                                    + new_resource_directory_size as usize)
                                    ..(old_resource_section.pointer_to_raw_data as usize
                                        + old_resource_section.size_of_raw_data as usize)],
                            );
                        } else if old_resource_section.size_of_raw_data
                            > new_resource_directory_size
                        {
                            debug!(
                                "resource section is the last section and smaller than the existing section, truncating section"
                            );
                            // adjust section size and virtual size header values
                            resource_dd.size = new_resource_directory_size;
                            old_resource_section.virtual_size = new_resource_directory_size;
                            old_resource_section.size_of_raw_data =
                                new_resource_directory_size_aligned;
                        }
                    } else {
                        debug!(
                            "resource section is the last section and larger than the existing section, expanding section"
                        );
                        resource_section_data =
                            resource_directory.build(old_resource_data_directory.virtual_address);
                        resource_dd.size = new_resource_directory_size;
                        // adjust section size and virtual size header values
                        old_resource_section.virtual_size = new_resource_directory_size;
                        old_resource_section.size_of_raw_data = new_resource_directory_size_aligned;
                        // pad the data to the size of the raw data
                        resource_section_data
                            .resize(new_resource_directory_size_aligned as usize, 0);
                    }
                } else {
                    debug!(
                        "resource section used by multiple data directories, keeping section intact"
                    );
                    warn!(
                        "resource section used by multiple data directories can indicate a packed executable"
                    );
                    add_new_section = true;
                }
            }
        }

        if add_new_section {
            debug!("adding new resource section");
            if let Some(ref mut old_resource_section) = old_resource_section {
                // copy existing resource section data that might be referenced by other data directories
                resource_section_data.extend(
                    &self.image[old_resource_section.pointer_to_raw_data as usize
                        ..(old_resource_section.pointer_to_raw_data
                            + old_resource_section.size_of_raw_data)
                            as usize],
                );
            }

            let virtual_address = {
                let last_virtual_section = section_table
                    .iter()
                    .max_by_key(|table| table.virtual_address + table.virtual_size);
                if let Some(last_virtual_section) = last_virtual_section {
                    last_virtual_section.virtual_address + last_virtual_section.virtual_size
                } else {
                    windows_header.section_alignment()
                }
            };
            let virtual_address = aligned_to(virtual_address, windows_header.section_alignment());

            let resource_dd = header_data_directory
                .get_mut(&DataDirectoryType::ResourceTable)
                .unwrap();
            resource_dd.virtual_address = virtual_address;
            resource_dd.size = new_resource_directory_size;

            let pointer_to_raw_data = {
                if let Some(last_section) = last_section {
                    last_section.pointer_to_raw_data + last_section.size_of_raw_data
                } else {
                    self.directories_offset as u32
                }
            };
            let new_section = SectionHeader {
                name: u64::from_le_bytes(".pedata\0".as_bytes().try_into().unwrap()),
                virtual_size: new_resource_directory_size,
                virtual_address,
                size_of_raw_data: new_resource_directory_size_aligned,
                pointer_to_raw_data,
                characteristics: IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
                ..SectionHeader::default()
            };
            section_table.push(new_section);
            new_section_data = resource_directory.build(virtual_address);

            coff_header.number_of_sections += 1;
            required_header_space += 40;
        }

        debug!("directories offset: {:#x?}", self.directories_offset);
        debug!(
            "first section start: {:#x?} ({})",
            first_section_start,
            first_section
                .and_then(|section| section.name())
                .unwrap_or("?".to_string())
        );
        let available_space = first_section_start - self.directories_offset as usize;
        debug!("available header space: {:#x?}", available_space);
        debug!(
            "required additional header space: {:#x?}",
            required_header_space
        );
        if required_header_space as usize > available_space {
            error!(
                "not enough space in header to add new section ({} > {})",
                required_header_space, available_space
            );
            return Err(ImageWriteError::NotEnoughSpaceInHeader);
        }

        match windows_header {
            GenericWindowsHeader::WindowsHeader32(ref mut header) => {
                header.number_of_rva_and_sizes = header_data_directory.len() as u32;
                header.size_of_image += new_section_data.len() as u32;
                header.check_sum = 0;
            }
            GenericWindowsHeader::WindowsHeader64(ref mut header) => {
                header.number_of_rva_and_sizes = header_data_directory.len() as u32;
                header.size_of_image += new_section_data.len() as u32;
                header.check_sum = 0;
            }
        }

        new_image.extend_from_slice(coff_header.as_bytes());
        new_image.extend_from_slice(self.standard_header.as_bytes());
        new_image.extend_from_slice(windows_header.as_bytes());

        for (_, data) in header_data_directory.iter() {
            new_image.extend_from_slice(data.as_bytes());
        }
        for section_header in section_table.iter() {
            new_image.extend_from_slice(section_header.as_bytes());
        }

        new_image.extend_from_slice(
            &self.image
                [(self.directories_offset + required_header_space) as usize..first_section_start],
        );

        if old_resource_section_start > 0 {
            // a resource section was found in the image, copy the data of sections around it
            new_image
                .extend_from_slice(&self.image[first_section_start..old_resource_section_start]);
            new_image.extend_from_slice(&resource_section_data);
            new_image.extend_from_slice(&self.image[old_resource_section_end..last_section_end]);
        } else {
            // no resource section was found in the image, copy data of all sections
            new_image.extend_from_slice(&self.image[first_section_start..last_section_end]);
        }
        new_image.extend_from_slice(&new_section_data);
        new_image.extend_from_slice(&self.image[last_section_end..]);

        self.coff_header = coff_header;
        self.windows_header = windows_header;
        self.header_data_directory = header_data_directory;
        self.section_table = section_table;

        let previous_resource_directory = self.resource_directory.take();
        self.resource_directory = Some(resource_directory);
        self.image = new_image.into();

        Ok(previous_resource_directory)
    }

    /// Set the subsystem running the image.
    /// This will update the subsystem field in the windows header.
    ///
    /// # Returns
    /// Returns the previous subsystem.
    pub fn set_subsystem(&mut self, subsystem: WORD) -> WORD {
        let previous_subsystem;
        match self.windows_header {
            GenericWindowsHeader::WindowsHeader32(ref mut header) => {
                previous_subsystem = header.subsystem;
                header.subsystem = subsystem;
                header.check_sum = 0;
            }
            GenericWindowsHeader::WindowsHeader64(ref mut header) => {
                previous_subsystem = header.subsystem;
                header.subsystem = subsystem;
                header.check_sum = 0;
            }
        }
        let mut new_image = Vec::with_capacity(self.image.len());
        new_image.extend_from_slice(&self.image[..self.coff_header_offset as usize]);
        new_image.extend_from_slice(self.coff_header.as_bytes());
        new_image.extend_from_slice(self.standard_header.as_bytes());
        new_image.extend_from_slice(self.windows_header.as_bytes());
        new_image.extend_from_slice(&self.image[self.optional_header_dd_offset as usize..]);
        self.image = new_image.into();
        previous_subsystem
    }

    /// Returns the current resource directory or `None` if the image does not contain a resource directory.
    pub fn resource_directory(&self) -> Option<&ResourceDirectory> {
        self.resource_directory.as_ref()
    }

    /// Returns the subsystem running the image.
    /// This will read the subsystem field in the windows header.
    pub fn subsystem(&self) -> WORD {
        match self.windows_header {
            GenericWindowsHeader::WindowsHeader32(ref header) => header.subsystem,
            GenericWindowsHeader::WindowsHeader64(ref header) => header.subsystem,
        }
    }

    /// Returns the raw image data with all changes applied.
    pub fn data(&self) -> &[u8] {
        &self.image
    }

    /// Returns the parsed coff header.
    pub fn coff_header(&self) -> &CoffHeader {
        &self.coff_header
    }

    /// Returns the parsed standard header.
    pub fn standard_header(&self) -> &StandardHeader {
        &self.standard_header
    }

    /// Returns the parsed windows header.
    pub fn windows_header(&self) -> &GenericWindowsHeader {
        &self.windows_header
    }

    /// Returns the data directory for the requested header.
    pub fn data_directory(&self, directory: DataDirectoryType) -> Option<&ImageDataDirectory> {
        self.header_data_directory.get(&directory)
    }

    /// Returns all data directories existing in the image.
    pub fn data_directories(&self) -> Vec<DataDirectoryType> {
        self.header_data_directory
            .keys()
            .copied()
            .collect::<Vec<_>>()
    }

    /// Returns the section header for the section at the index.
    pub fn section_header<Index: Into<usize>>(&self, index: Index) -> Option<&SectionHeader> {
        self.section_table.get(index.into())
    }

    /// Returns the section header containing the data directory.
    pub fn section_header_for_data_directory(
        &self,
        directory: DataDirectoryType,
    ) -> Option<&SectionHeader> {
        if let Some(data_directory) = self.data_directory(directory) {
            for section_table in self.section_table.iter() {
                if data_directory.virtual_address >= section_table.virtual_address
                    && data_directory.virtual_address
                        < section_table.virtual_address + section_table.virtual_size
                {
                    return Some(section_table);
                }
            }
        }
        None
    }

    /// Returns all section tables existing in the image.
    pub fn section_table(&self) -> &Vec<SectionHeader> {
        &self.section_table
    }

    /// Returns the `Image` with all data cloned into owned memory.
    pub fn cloned(&self) -> Image<'static> {
        Image {
            image: self.image.clone().into_owned().into(),
            pe_dos_magic: self.pe_dos_magic,
            pe_signature: self.pe_signature,
            coff_header: self.coff_header,
            standard_header: self.standard_header,
            windows_header: self.windows_header,
            header_data_directory: self.header_data_directory.clone(),
            section_table: self.section_table.clone(),
            resource_directory: self.resource_directory.clone(),
            coff_header_offset: self.coff_header_offset,
            optional_header_dd_offset: self.optional_header_dd_offset,
            directories_offset: self.directories_offset,
        }
    }
}
