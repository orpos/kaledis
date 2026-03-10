//! Data types for parsing and building the resource section.
//! The resource section contains the resource directory and the resource data.
//! See <https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#the-rsrc-section> for more information.

use core::{borrow::Borrow, iter, mem::size_of};

use ahash::RandomState;
use debug_ignore::DebugIgnore;
use image::DynamicImage;
use indexmap::{IndexMap, IndexSet};
use log::{error, trace, warn};
use zerocopy::IntoBytes;

/// Trait for data types that can be converted to icon data.
///
/// This trait is implemented for `&[u8]`, `Vec<u8>`, and for `DynamicImage` when the `images` feature is enabled.
pub trait ToIcon {
    fn icons(&self) -> Result<Vec<Vec<u8>>, ResourceError>;
}
impl ToIcon for &[u8] {
    fn icons(&self) -> Result<Vec<Vec<u8>>, ResourceError> {
        if self.len() < 22 {
            return Err(ResourceError::InvalidBytes("icon data is too small".into()));
        }
        let directory = read::<IconDirectory>(&self[0..6])?;
        if directory.type_ != 1 {
            return Err(ResourceError::InvalidBytes(
                "icon data is not an icon".into(),
            ));
        }
        if directory.count < 1 {
            return Err(ResourceError::InvalidBytes(
                "icon data has no images".into(),
            ));
        }
        let mut icons = Vec::with_capacity(directory.count as usize);
        for i in 0..directory.count as usize {
            if self.len() < 6 + i * 16 + 16 {
                return Err(ResourceError::InvalidBytes("icon data is too small".into()));
            }
            let size = read::<u32>(&self[6..][i * 16 + 8..])? as usize;
            let offset = read::<u32>(&self[6..][i * 16 + 12..])? as usize;
            if offset + size > self.len() {
                return Err(ResourceError::InvalidBytes("icon data is truncated".into()));
            }
            let mut data = Vec::new();
            data.extend(&self[offset..offset + size]);
            icons.push(data);
        }
        Ok(icons)
    }
}
impl ToIcon for Vec<u8> {
    fn icons(&self) -> Result<Vec<Vec<u8>>, ResourceError> {
        self.as_slice().icons()
    }
}
impl ToIcon for &DynamicImage {
    fn icons(&self) -> Result<Vec<Vec<u8>>, ResourceError> {
        use image::{ImageFormat, imageops::FilterType::Lanczos3};
        use std::io::Cursor;
        const RESOLUTIONS: &[u32] = &[256, 128, 48, 32, 24, 16];
        RESOLUTIONS
            .iter()
            .map(|&size| {
                let mut data = Vec::new();
                self.resize_exact(size, size, Lanczos3)
                    .to_rgba8()
                    .write_to(&mut Cursor::new(&mut data), ImageFormat::Ico)?;
                Ok(data)
            })
            .collect::<Result<Vec<Vec<u8>>, ResourceError>>()
    }
}
impl ToIcon for DynamicImage {
    fn icons(&self) -> Result<Vec<Vec<u8>>, ResourceError> {
        (&self).icons()
    }
}

use super::{constants::*, errors::*, types::*, util::*};

/// Portable executable resource directory.
///
/// The resource directory contains the resource table and the resource data entries.
///
/// See [`Image::resource_directory`](crate::Image::resource_directory) for retrieving the resource directory from an image.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ResourceDirectory {
    pub(crate) virtual_address: u32,
    pub(crate) root: ResourceTable,
}
impl ResourceDirectory {
    /// Parse the resource directory from the given image at the given base address.
    /// The virtual address is used to resolve the resource data offsets and has to correspond to the virtual address in the section table header of the source image.
    ///
    /// # Returns
    /// Returns an error if the resource directory at the given address is invalid.
    pub fn parse(
        image: &[u8],
        base_address: u32,
        virtual_address: u32,
    ) -> Result<Self, ImageReadError> {
        let root = ResourceTable::parse(image, base_address, virtual_address, 0, 0)?;
        Ok(Self {
            virtual_address,
            root,
        })
    }

    /// Get the main icon of the executable.
    /// The icon will be the first icon in the `MAINICON` group icon directory if it exists.
    /// Otherwise, the first icon in the first group icon directory will be returned.
    ///
    /// # Returns
    /// Returns `None` if no icon exists.
    /// Returns an error if the resource table structure is not well-formed.
    pub fn get_main_icon(&self) -> Result<Option<&[u8]>, ResourceError> {
        if self.root.entries.is_empty() {
            return Ok(None);
        }

        // find the group icon table
        let group_table = self.root.get(ResourceEntryName::ID(RT_GROUP_ICON as u32));
        let group_table = match group_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "group icon table is not a table".to_string(),
                ));
            }
            _ => return Ok(None),
        };
        if group_table.entries.is_empty() {
            return Ok(None);
        }

        // find the main icon directory table
        let icon_directory_table = group_table
            .entries
            .get(&ResourceEntryName::from_string("MAINICON"))
            .or_else(|| group_table.entries.first().map(|(_, v)| v));
        let icon_directory_table = match icon_directory_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner group icon table is not a table".to_string(),
                ));
            }
            None => return Ok(None),
        };
        if icon_directory_table.entries.is_empty() {
            return Ok(None);
        }

        // find the main icon directory
        let icon_directory_entry = icon_directory_table
            .entries
            .first()
            .map(|(_, v)| v)
            .unwrap();
        if icon_directory_entry.is_table() {
            return Err(ResourceError::InvalidTable(
                "group icon table entry is not data".to_string(),
            ));
        }
        let icon_directory_entry = icon_directory_entry.as_data().unwrap();
        let icon_directory = read::<IconDirectory>(&icon_directory_entry.data)?;

        // get the first icon in the main icon directory
        if icon_directory.count == 0 {
            return Ok(None);
        }
        let icon_directory_entry = read::<IconDirectoryEntry>(&icon_directory_entry.data[6..])?;
        let icon_id = icon_directory_entry.id as u32;

        // find the main icon table
        let icon_table = self.root.get(ResourceEntryName::ID(RT_ICON as u32));
        if icon_table.is_none() {
            return Ok(None);
        }
        let icon_table = match icon_table.unwrap() {
            ResourceEntry::Table(table) => table,
            ResourceEntry::Data(_) => {
                return Err(ResourceError::InvalidTable(
                    "icon table is not a table".to_string(),
                ));
            }
        };

        let inner_table = icon_table.get(ResourceEntryName::ID(icon_id));
        let inner_table = match inner_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner icon table is not a table".to_string(),
                ));
            }
            None => return Ok(None),
        };
        if inner_table.entries.is_empty() {
            return Ok(None);
        }

        // get the main icon from the icon table
        let icon = match inner_table.entries.first().map(|(_, v)| v) {
            Some(ResourceEntry::Table(_)) => {
                return Err(ResourceError::InvalidTable(
                    "icon table entry is not data".to_string(),
                ));
            }
            Some(ResourceEntry::Data(data)) => data,
            None => return Ok(None),
        };

        Ok(Some(icon.data()))
    }

    /// Set the main icon of the executable.
    /// The icon must be the byte slice of a valid icon, or a [`image::DynamicImage`] when the `images` feature is enabled.
    ///
    /// When `icon` is a [`image::DynamicImage`], the image is resized to the different icon resolutions.
    ///
    /// This will overwrite the group icon directory with the `MAINICON` name if it exists and keep all other group icon directories intact.
    /// This will not remove any existing icons.
    /// To remove the existing main icon directory and the icons referenced by, call [`remove_main_icon`](ResourceDirectory::remove_main_icon) before setting a new one.
    ///
    /// # Returns
    /// Returns an error if the new icon not a valid image or the resource table structure is not well-formed.
    pub fn set_main_icon<T: ToIcon>(&mut self, icon: T) -> Result<(), ResourceError> {
        // find the main icon table
        if self
            .root
            .get(ResourceEntryName::ID(RT_ICON as u32))
            .is_none()
        {
            self.root.insert(
                ResourceEntryName::ID(RT_ICON as u32),
                ResourceEntry::Table(ResourceTable::default()),
            );
        }
        let icon_table = match self
            .root
            .get_mut(ResourceEntryName::ID(RT_ICON as u32))
            .unwrap()
        {
            ResourceEntry::Table(table) => table,
            ResourceEntry::Data(_) => {
                return Err(ResourceError::InvalidTable(
                    "icon table is not a table".to_string(),
                ));
            }
        };

        // find the first free icon id
        let first_free_icon_id = icon_table
            .entries
            .keys()
            .filter_map(|k| match k {
                ResourceEntryName::ID(id) => Some(*id),
                _ => None,
            })
            .max()
            .unwrap_or(0)
            + 1;

        // read the icon and resize it to the different resolutions
        let icons = icon.icons()?;

        // add the icons to the icon table
        let mut icon_directory_entries = Vec::new();
        for (i, icon) in icons.iter().enumerate() {
            let id = first_free_icon_id + i as u32;
            let mut inner_table = ResourceTable::default();
            inner_table.insert(
                ResourceEntryName::ID(LANGUAGE_ID_EN_US as u32),
                ResourceEntry::Data(ResourceData {
                    data: {
                        let mut entry = read::<IconDirectoryEntry>(&icon[6..20])?;
                        entry.id = id as u16;
                        icon_directory_entries.push(entry);
                        icon[22..].to_owned().into()
                    },
                    codepage: CODE_PAGE_ID_EN_US as u32,
                    reserved: 0,
                }),
            );
            icon_table.insert(ResourceEntryName::ID(id), ResourceEntry::Table(inner_table));
        }

        // find the group icon table
        if self
            .root
            .get(ResourceEntryName::ID(RT_GROUP_ICON as u32))
            .is_none()
        {
            self.root.insert(
                ResourceEntryName::ID(RT_GROUP_ICON as u32),
                ResourceEntry::Table(ResourceTable::default()),
            );
        }
        let group_table = match self
            .root
            .get_mut(ResourceEntryName::ID(RT_GROUP_ICON as u32))
            .unwrap()
        {
            ResourceEntry::Table(table) => table,
            ResourceEntry::Data(_) => {
                return Err(ResourceError::InvalidTable(
                    "group icon table is not a table".to_string(),
                ));
            }
        };

        // insert the main icon directory table
        let mut inner_table = ResourceTable::default();
        inner_table.insert(
            ResourceEntryName::ID(LANGUAGE_ID_EN_US as u32),
            ResourceEntry::Data(ResourceData {
                data: {
                    let mut data = Vec::new();
                    let icon_directory = IconDirectory {
                        reserved: 0,
                        type_: 1,
                        count: icon_directory_entries.len() as u16,
                    };
                    data.extend(icon_directory.as_bytes());
                    for entry in icon_directory_entries {
                        data.extend(&entry.as_bytes()[..14]);
                    }
                    data.into()
                },
                codepage: CODE_PAGE_ID_EN_US as u32,
                reserved: 0,
            }),
        );
        group_table.insert_at(
            ResourceEntryName::from_string("MAINICON"),
            ResourceEntry::Table(inner_table),
            0,
        );

        Ok(())
    }

    /// Set the main icon of the executable from a file.
    /// The file must contain a valid image.
    /// The image is resized to the different icon resolutions when the `images` feature is enabled.
    ///
    /// See [`set_main_icon`](ResourceDirectory::set_main_icon) for more information.
    ///
    /// # Returns
    /// Returns an error if the file is not a valid image or the resource table structure is not well-formed.
    pub fn set_main_icon_file(&mut self, path: &str) -> Result<(), ResourceError> {
        let icon = image::ImageReader::open(path)?.decode()?;
        self.set_main_icon(icon)
    }

    /// Set the main icon of the executable from a reader.
    /// The reader must contain a valid image.
    /// The image is resized to the different icon resolutions when the `images` feature is enabled.
    ///
    /// See [`set_main_icon`](ResourceDirectory::set_main_icon) for more information.
    ///
    /// # Returns
    /// Returns an error if the reader does not contain a valid image or the resource table structure is not well-formed.
    pub fn set_main_icon_reader<R: std::io::Read>(
        &mut self,
        reader: &mut R,
    ) -> Result<(), ResourceError> {
        let mut icon = Vec::new();
        reader.read_to_end(&mut icon)?;
        let icon = image::load_from_memory(&icon)?;
        self.set_main_icon(icon)
    }

  

    /// Remove the main icon directory and all icons uniquely referenced by it.
    ///
    /// # Returns
    /// Returns an error if the icon resource directory is invalid.
    pub fn remove_main_icon(&mut self) -> Result<(), ResourceError> {
        if self.root.entries.is_empty() {
            return Ok(());
        }

        // find the group table
        let group_table = self
            .root
            .get_mut(ResourceEntryName::ID(RT_GROUP_ICON as u32));
        let group_table = match group_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "group icon table is not a table".to_string(),
                ));
            }
            _ => return Ok(()),
        };
        if group_table.entries.is_empty() {
            return Ok(());
        }

        // find the main icon directory table
        let mut icon_directory_name = ResourceEntryName::from_string("MAINICON");
        let mut icon_directory_table = group_table.get(&icon_directory_name);
        if icon_directory_table.is_none() {
            icon_directory_table = group_table.entries.first().map(|(name, v)| {
                icon_directory_name = name.clone();
                v
            });
        }
        let icon_directory_table = match icon_directory_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner group icon table is not a table".to_string(),
                ));
            }
            _ => return Ok(()),
        };
        if icon_directory_table.entries.is_empty() {
            return Ok(());
        }

        // find the main icon directory
        let icon_directory_entry = icon_directory_table
            .entries
            .first()
            .map(|(_, v)| v)
            .unwrap();
        if icon_directory_entry.is_table() {
            return Err(ResourceError::InvalidTable(
                "group icon table entry is not data".to_string(),
            ));
        }
        let icon_directory_entry = icon_directory_entry.as_data().unwrap();
        let icon_directory = read::<IconDirectory>(&icon_directory_entry.data)?;

        // get a list of all icons in the main icon directory for removal
        let mut icons_to_remove = IndexSet::with_hasher(RandomState::default());
        for i in 0..icon_directory.count {
            let icon_directory_entry = read::<IconDirectoryEntry>(
                &icon_directory_entry.data[6 + i as usize * size_of::<IconDirectoryEntry>()..],
            )?;
            let icon_id = icon_directory_entry.id;
            icons_to_remove.insert(icon_id);
        }

        // get a list of icons in other icon directories and remove them from the list
        for (other_icon_directory_name, other_icon_directory_table) in group_table.entries.iter() {
            if other_icon_directory_name == &icon_directory_name {
                continue;
            }
            if !other_icon_directory_table.is_table() {
                continue;
            }
            let other_icon_directory_table = other_icon_directory_table.as_table().unwrap();
            if other_icon_directory_table.entries.is_empty() {
                continue;
            }
            let other_icon_directory_entry = other_icon_directory_table
                .entries
                .first()
                .map(|(_, v)| v)
                .unwrap();
            if other_icon_directory_entry.is_table() {
                continue;
            }
            let other_icon_directory_entry = other_icon_directory_entry.as_data().unwrap();
            let other_icon_directory = read::<IconDirectory>(&other_icon_directory_entry.data)?;
            for i in 0..other_icon_directory.count {
                let icon_directory_entry = read::<IconDirectoryEntry>(
                    &other_icon_directory_entry.data
                        [6 + i as usize * size_of::<IconDirectoryEntry>()..],
                )?;
                let icon_id = icon_directory_entry.id;
                icons_to_remove.swap_remove(&icon_id);
            }
        }

        // remove the main icon directory table
        group_table.remove(&icon_directory_name);
        if group_table.entries.is_empty() {
            self.root
                .remove(ResourceEntryName::ID(RT_GROUP_ICON as u32));
        }

        // find the main icon table
        let icon_table = self.root.get_mut(ResourceEntryName::ID(RT_ICON as u32));
        if icon_table.is_none() {
            return Ok(());
        }
        let icon_table = icon_table.unwrap();
        if !icon_table.is_table() {
            return Ok(());
        }
        let icon_table = icon_table.as_table_mut().unwrap();

        // remove the icons from the icon table
        for icon_id in icons_to_remove {
            icon_table.remove(ResourceEntryName::ID(icon_id as u32));
        }

        Ok(())
    }

    /// Get the version information of the executable.
    ///
    /// # Returns
    /// Returns `None` if no version information exists.
    /// Returns an error if the version resource directory is invalid.
    pub fn get_version_info(&self) -> Result<Option<VersionInfo>, ResourceError> {
        if self.root.entries.is_empty() {
            return Ok(None);
        }

        // find the group table
        let version_table = self.root.get(ResourceEntryName::ID(RT_VERSION as u32));
        let version_table = match version_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "version table is not a table".to_string(),
                ));
            }
            _ => return Ok(None),
        };
        if version_table.entries.is_empty() {
            return Ok(None);
        }

        // find the main version directory table
        let inner_table = version_table.entries.first().map(|(_, v)| v);
        let inner_table = match inner_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner version table is not a table".to_string(),
                ));
            }
            None => return Ok(None),
        };
        if inner_table.entries.is_empty() {
            return Ok(None);
        }

        // find the main version directory
        let version_directory_entry = inner_table
            .entries
            .iter()
            .find(|(name, _)| **name == ResourceEntryName::ID(LANGUAGE_ID_EN_US as u32))
            .or_else(|| inner_table.entries.first())
            .map(|(_, v)| v)
            .unwrap();
        if version_directory_entry.is_table() {
            return Err(ResourceError::InvalidTable(
                "version table entry is not data".to_string(),
            ));
        }
        let version_directory_entry = version_directory_entry.as_data().unwrap();

        Ok(Some(VersionInfo::parse(&version_directory_entry.data)?))
    }

    /// Set the version information of the executable.
    ///
    /// This will overwrite the existing version information.
    ///
    /// # Returns
    /// Returns an error if the resource table structure is not well-formed.
    pub fn set_version_info(&mut self, version_info: &VersionInfo) -> Result<(), ResourceError> {
        // find the version table
        if self
            .root
            .get(ResourceEntryName::ID(RT_VERSION as u32))
            .is_none()
        {
            self.root.insert(
                ResourceEntryName::ID(RT_VERSION as u32),
                ResourceEntry::Table(ResourceTable::default()),
            );
        }
        let version_table = match self
            .root
            .get_mut(ResourceEntryName::ID(RT_VERSION as u32))
            .unwrap()
        {
            ResourceEntry::Table(table) => table,
            ResourceEntry::Data(_) => {
                return Err(ResourceError::InvalidTable(
                    "version table is not a table".to_string(),
                ));
            }
        };

        // find the main version directory table
        let inner_table = version_table.entries.first().map(|(_, v)| v);
        let mut inner_table = match inner_table {
            Some(ResourceEntry::Table(t)) => t.clone(),
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner version table is not a table".to_string(),
                ));
            }
            None => ResourceTable::default(),
        };

        inner_table.insert_at(
            ResourceEntryName::ID(LANGUAGE_ID_EN_US as u32),
            ResourceEntry::Data(ResourceData {
                data: version_info.build().into(),
                codepage: CODE_PAGE_ID_EN_US as u32,
                reserved: 0,
            }),
            0,
        );
        version_table.insert_at(
            ResourceEntryName::ID(1),
            ResourceEntry::Table(inner_table),
            0,
        );

        Ok(())
    }

    /// Remove the version information of the executable.
    ///
    /// # Returns
    /// Returns an error if the resource table structure is not well-formed.
    pub fn remove_version_info(&mut self) -> Result<(), ResourceError> {
        if self.root.entries.is_empty() {
            return Ok(());
        }

        // find the version table
        let version_table = self.root.get_mut(ResourceEntryName::ID(RT_VERSION as u32));
        let version_table = match version_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "version table is not a table".to_string(),
                ));
            }
            _ => return Ok(()),
        };
        if version_table.entries.is_empty() {
            return Ok(());
        }

        // find the main version directory table
        let inner_table = version_table.entries.first_mut().map(|(_, v)| v);
        let inner_table = match inner_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner version table is not a table".to_string(),
                ));
            }
            None => return Ok(()),
        };
        if inner_table.entries.is_empty() {
            return Ok(());
        }

        // remove the main version directory
        inner_table.remove(inner_table.entries.keys().next().unwrap().clone());
        if inner_table.entries.is_empty() {
            version_table.remove(version_table.entries.keys().next().unwrap().clone());
        }
        if version_table.entries.is_empty() {
            self.root.remove(ResourceEntryName::ID(RT_VERSION as u32));
        }

        Ok(())
    }

    /// Get the manifest of the executable.
    ///
    /// # Returns
    /// Returns `None` if no manifest exists.
    /// Returns an error if the manifest resource directory is invalid.
    pub fn get_manifest(&self) -> Result<Option<String>, ResourceError> {
        if self.root.entries.is_empty() {
            return Ok(None);
        }

        // find the manifest table
        let manifest_table = self.root.get(ResourceEntryName::ID(RT_MANIFEST as u32));
        let manifest_table = match manifest_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "manifest table is not a table".to_string(),
                ));
            }
            _ => return Ok(None),
        };
        if manifest_table.entries.is_empty() {
            return Ok(None);
        }

        // find the main manifest directory table
        let inner_table = manifest_table.entries.first().map(|(_, v)| v);
        let inner_table = match inner_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner manifest table is not a table".to_string(),
                ));
            }
            None => return Ok(None),
        };
        if inner_table.entries.is_empty() {
            return Ok(None);
        }

        // find the main manifest directory
        let manifest_directory_entry = inner_table
            .entries
            .iter()
            .find(|(name, _)| **name == ResourceEntryName::ID(LANGUAGE_ID_EN_US as u32))
            .or_else(|| inner_table.entries.first())
            .map(|(_, v)| v)
            .unwrap();
        if manifest_directory_entry.is_table() {
            return Err(ResourceError::InvalidTable(
                "manifest table entry is not data".to_string(),
            ));
        }
        let manifest_directory_entry = manifest_directory_entry.as_data().unwrap();

        Ok(Some(
            String::from_utf8_lossy(&manifest_directory_entry.data).to_string(),
        ))
    }

    /// Set the manifest of the executable.
    ///
    /// This will overwrite the existing manifest.
    ///
    /// # Returns
    /// Returns an error if the resource table structure is not well-formed.
    pub fn set_manifest(&mut self, manifest: &str) -> Result<(), ResourceError> {
        if self.root.entries.is_empty() {
            return Ok(());
        }

        if self
            .root
            .get(ResourceEntryName::ID(RT_MANIFEST as u32))
            .is_none()
        {
            self.root.insert(
                ResourceEntryName::ID(RT_MANIFEST as u32),
                ResourceEntry::Table(ResourceTable::default()),
            );
        }
        let manifest_table = match self
            .root
            .get_mut(ResourceEntryName::ID(RT_MANIFEST as u32))
            .unwrap()
        {
            ResourceEntry::Table(table) => table,
            ResourceEntry::Data(_) => {
                return Err(ResourceError::InvalidTable(
                    "manifest table is not a table".to_string(),
                ));
            }
        };

        // find the main manifest directory table
        let inner_table = manifest_table.entries.first().map(|(_, v)| v);
        let mut inner_table = match inner_table {
            Some(ResourceEntry::Table(t)) => t.clone(),
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner manifest table is not a table".to_string(),
                ));
            }
            None => ResourceTable::default(),
        };

        inner_table.insert_at(
            ResourceEntryName::ID(LANGUAGE_ID_EN_US as u32),
            ResourceEntry::Data(ResourceData {
                data: manifest.as_bytes().to_vec().into(),
                codepage: CODE_PAGE_ID_EN_US as u32,
                reserved: 0,
            }),
            0,
        );
        manifest_table.insert_at(
            ResourceEntryName::ID(1),
            ResourceEntry::Table(inner_table),
            0,
        );

        Ok(())
    }

    /// Remove the manifest of the executable.
    ///
    /// # Returns
    /// Returns an error if the resource table structure is not well-formed.
    pub fn remove_manifest(&mut self) -> Result<(), ResourceError> {
        if self.root.entries.is_empty() {
            return Ok(());
        }

        // find the version table
        let manifest_table = self.root.get_mut(ResourceEntryName::ID(RT_MANIFEST as u32));
        let manifest_table = match manifest_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "manifest table is not a table".to_string(),
                ));
            }
            _ => return Ok(()),
        };
        if manifest_table.entries.is_empty() {
            return Ok(());
        }

        // find the main manifest directory table
        let inner_table = manifest_table.entries.first_mut().map(|(_, v)| v);
        let inner_table = match inner_table {
            Some(ResourceEntry::Table(t)) => t,
            Some(_) => {
                return Err(ResourceError::InvalidTable(
                    "inner manifest table is not a table".to_string(),
                ));
            }
            None => return Ok(()),
        };
        if inner_table.entries.is_empty() {
            return Ok(());
        }

        // remove the main manifest directory
        inner_table.remove(inner_table.entries.keys().next().unwrap().clone());
        if inner_table.entries.is_empty() {
            manifest_table.remove(manifest_table.entries.keys().next().unwrap().clone());
        }
        if manifest_table.entries.is_empty() {
            self.root.remove(ResourceEntryName::ID(RT_MANIFEST as u32));
        }

        Ok(())
    }

    /// Returns the virtual address of the resource directory in the source image.
    pub fn virtual_address(&self) -> u32 {
        self.virtual_address
    }

    /// Returns the root resource table.
    /// The root resource table contains the top-level resource entries.
    pub fn root(&self) -> &ResourceTable {
        &self.root
    }

    /// Returns the mutable root resource table.
    /// The root resource table contains the top-level resource entries.
    pub fn root_mut(&mut self) -> &mut ResourceTable {
        &mut self.root
    }

    /// Returns the size of the resulting resource directory in bytes.
    pub fn size(&self) -> u32 {
        self.root.size()
    }

    /// Build the resource directory into raw bytes to be included in an image.
    /// The virtual address is used to compute the resource data offsets and has to correspond to the virtual address in the section table header of the target image.
    pub fn build(&self, virtual_address: u32) -> Vec<u8> {
        self.root.build(virtual_address)
    }
}

/// Portable executable resource table data.
enum TableData {
    Table(ResourceDirectoryTable),
    Entry(ResourceDirectoryEntry),
}

/// Portable executable resource table.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ResourceTable {
    pub(crate) data: ResourceDirectoryTable,
    pub(crate) entries: IndexMap<ResourceEntryName, ResourceEntry, RandomState>,
}
impl ResourceTable {
    fn parse(
        image: &[u8],
        base_address: u32,
        virtual_address: u32,
        directory_offset: u32,
        level: usize,
    ) -> Result<Self, ImageReadError> {
        let table_offset = base_address + directory_offset;
        let resource_table = read::<ResourceDirectoryTable>(&image[table_offset as usize..])?;
        trace!("{} {:#x?}", "--".repeat(level + 1), resource_table);

        let mut entries = IndexMap::default();

        let mut entry_offset = table_offset + 16;
        for _ in 0..(resource_table.number_of_name_entries + resource_table.number_of_id_entries) {
            let entry = read::<ResourceDirectoryEntry>(&image[entry_offset as usize..])?;
            trace!("{} {:#x?}", "--".repeat(level + 1), entry);

            if entry.data_entry_or_subdirectory_offset & 0x80000000 != 0 {
                entries.insert(
                    ResourceEntryName::parse(image, base_address, entry.name_offset_or_integer_id)?,
                    ResourceEntry::Table(ResourceTable::parse(
                        image,
                        base_address,
                        virtual_address,
                        entry.data_entry_or_subdirectory_offset ^ 0x80000000,
                        level + 1,
                    )?),
                );
            } else {
                trace!(
                    "reading {} bytes at {} (image size {})",
                    size_of::<ResourceDataEntry>(),
                    base_address + entry.data_entry_or_subdirectory_offset,
                    image.len()
                );
                let data = read::<ResourceDataEntry>(
                    &image[(base_address + entry.data_entry_or_subdirectory_offset) as usize..],
                )?;
                // calculate as i64 and convert to u64 first to check for padding
                let address = base_address as i64 + data.data_rva as i64 - virtual_address as i64;
                let mut address = address as u64;
                if address & 0xffffffffff000000 == 0xffffffffff000000 {
                    warn!(
                        "{} resource data entry address {:#x?} seems to be packed, ignoring padding",
                        "--".repeat(level + 1),
                        address
                    );
                    address ^= 0xffffffffff000000;
                }
                trace!("{} {:#x?} {:#x?}", "--".repeat(level + 1), address, data);
                if address + data.size as u64 > image.len() as u64 {
                    error!(
                        "{} resource data entry address {:#x?} with size {:#x?} ({:#x?}) outside valid range ({:#x?})",
                        "--".repeat(level + 1),
                        address,
                        data.size,
                        address + data.size as u64,
                        image.len()
                    );
                    continue;
                }
                let address = address as u32;
                entries.insert(
                    ResourceEntryName::parse(image, base_address, entry.name_offset_or_integer_id)?,
                    ResourceEntry::Data(ResourceData {
                        codepage: data.codepage,
                        reserved: data.reserved,
                        data: Vec::from(&image[address as usize..(address + data.size) as usize])
                            .into(),
                    }),
                );
            }

            entry_offset += 8;
        }
        Ok(Self {
            data: resource_table,
            entries,
        })
    }

    fn build(&self, virtual_address: u32) -> Vec<u8> {
        let mut tables_offset = 0;
        let mut strings_offset = 0;
        let mut descriptions_offset = 0;
        let mut data_offset = 0;
        let (mut tables_data, strings_data, mut descriptions_data, data_data) = self.build_table(
            virtual_address,
            &mut tables_offset,
            &mut strings_offset,
            &mut descriptions_offset,
            &mut data_offset,
        );

        let mut data = Vec::new();
        data.extend(tables_data.iter_mut().flat_map(|data| match data {
            TableData::Table(table) => table.as_bytes(),
            TableData::Entry(entry) => {
                if entry.data_entry_or_subdirectory_offset & 0x80000000 == 0 {
                    entry.data_entry_or_subdirectory_offset += tables_offset + strings_offset;
                }
                if entry.name_offset_or_integer_id & 0x80000000 != 0 {
                    entry.name_offset_or_integer_id += tables_offset;
                }
                entry.as_bytes()
            }
        }));
        data.extend(strings_data.iter());
        data.extend(descriptions_data.iter_mut().flat_map(|data| {
            data.data_rva += tables_offset + strings_offset + descriptions_offset;
            data.as_bytes()
        }));
        data.extend(data_data);

        data
    }

    fn build_table(
        &self,
        virtual_address: u32,
        tables_offset: &mut u32,
        strings_offset: &mut u32,
        descriptions_offset: &mut u32,
        data_offset: &mut u32,
    ) -> (Vec<TableData>, Vec<u8>, Vec<ResourceDataEntry>, Vec<u8>) {
        let mut tables_data = Vec::<TableData>::new();
        let mut strings_data = Vec::<u8>::new();
        let mut descriptions_data = Vec::<ResourceDataEntry>::new();
        let mut data_data = Vec::<u8>::new();

        tables_data.push(TableData::Table(self.data));
        *tables_offset += 16;

        let mut next_table_offset = 0u32;
        let mut next_table_sizes = 0u32;
        for (name, entry) in &self.entries {
            strings_data.extend(name.string_data());
            let name_offset_or_integer_id = if name.string_size() > 0 {
                *strings_offset | 0x80000000
            } else {
                name.id()
            };
            *strings_offset += name.string_size();

            match entry {
                ResourceEntry::Table(table) => {
                    let entry_data = ResourceDirectoryEntry {
                        name_offset_or_integer_id,
                        data_entry_or_subdirectory_offset: (*tables_offset
                            + self.entries.len() as u32 * 8
                            + next_table_sizes)
                            | 0x80000000,
                    };
                    tables_data.push(TableData::Entry(entry_data));
                    next_table_offset += 8;
                    next_table_sizes += table.tables_size();
                }
                ResourceEntry::Data(data) => {
                    let entry_data = ResourceDirectoryEntry {
                        name_offset_or_integer_id,
                        data_entry_or_subdirectory_offset: *descriptions_offset,
                    };
                    tables_data.push(TableData::Entry(entry_data));
                    next_table_offset += 8;

                    data_data.extend(&*data.data);
                    let description_data = ResourceDataEntry {
                        data_rva: *data_offset + virtual_address,
                        size: data.data.len() as u32,
                        codepage: data.codepage,
                        reserved: data.reserved,
                    };
                    descriptions_data.push(description_data);
                    *descriptions_offset += 16;
                    *data_offset += data.data.len() as u32;
                }
            }
        }
        *tables_offset += next_table_offset;

        for (_, entry) in &self.entries {
            match entry {
                ResourceEntry::Table(table) => {
                    let (t_tables_data, t_strings_data, t_descriptions_data, t_data_data) = table
                        .build_table(
                            virtual_address,
                            tables_offset,
                            strings_offset,
                            descriptions_offset,
                            data_offset,
                        );
                    tables_data.extend(t_tables_data);
                    strings_data.extend(t_strings_data);
                    descriptions_data.extend(t_descriptions_data);
                    data_data.extend(t_data_data);
                }
                ResourceEntry::Data(_) => {}
            }
        }

        (tables_data, strings_data, descriptions_data, data_data)
    }

    /// Get a resource entry from the table.
    /// # Returns
    /// The resource entry.
    pub fn get<N: Borrow<ResourceEntryName>>(&self, name: N) -> Option<&ResourceEntry> {
        self.entries.get(name.borrow())
    }

    /// Get a mutable resource entry from the table.
    /// # Returns
    /// The resource entry.
    pub fn get_mut<N: Borrow<ResourceEntryName>>(&mut self, name: N) -> Option<&mut ResourceEntry> {
        self.entries.get_mut(name.borrow())
    }

    /// Insert a resource entry into the table.
    /// If an entry with the given name already exists, it will be replaced.
    /// # Returns
    /// The replaced entry.
    pub fn insert<N: Borrow<ResourceEntryName>>(
        &mut self,
        name: N,
        entry: ResourceEntry,
    ) -> Option<ResourceEntry> {
        let name = name.borrow();
        let entry = self.entries.insert(name.clone(), entry);
        if entry.is_none() {
            if name.string_size() > 0 {
                self.data.number_of_name_entries += 1;
            } else {
                self.data.number_of_id_entries += 1;
            }
        }
        entry
    }

    /// Insert a resource entry into the table at the specified position.
    /// If an entry with the given name already exists, it will be replaced.
    /// # Returns
    /// The replaced entry.
    pub fn insert_at<N: Borrow<ResourceEntryName>>(
        &mut self,
        name: N,
        entry: ResourceEntry,
        position: usize,
    ) -> Option<ResourceEntry> {
        let name = name.borrow();
        let len = self.entries.len();
        let old_entry = self.entries.get(name).cloned();
        let new_entry = self
            .entries
            .entry(name.clone())
            .and_modify(|old_entry| *old_entry = entry.clone());
        let index = new_entry.index();
        new_entry.or_insert(entry);
        self.entries.move_index(index, position);
        if index >= len {
            if name.string_size() > 0 {
                self.data.number_of_name_entries += 1;
            } else {
                self.data.number_of_id_entries += 1;
            }
        }
        old_entry
    }

    /// Remove a resource entry from the table.
    /// # Returns
    /// The removed entry.
    pub fn remove<N: Borrow<ResourceEntryName>>(&mut self, name: N) -> Option<ResourceEntry> {
        let name = name.borrow();
        if let Some(entry) = self.entries.swap_remove(name) {
            if name.string_size() > 0 {
                self.data.number_of_name_entries -= 1;
            } else {
                self.data.number_of_id_entries -= 1;
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Returns the entries in the table.
    pub fn entries(&self) -> Vec<&ResourceEntryName> {
        self.entries.keys().collect()
    }

    /// Returns the complete size of the table, its resources and its children in the resource table.
    pub fn size(&self) -> u32 {
        self.tables_size() + self.strings_size() + self.descriptions_size() + self.data_size()
    }

    /// Returns the size of the table and its children in the resource table.
    pub fn tables_size(&self) -> u32 {
        self.entries
            .iter()
            .map(|(_, entry)| entry.table_size())
            .sum::<u32>()
            + 16
    }

    /// Returns the size of the strings in the entry and its children in the resource table.
    pub fn strings_size(&self) -> u32 {
        self.entries
            .iter()
            .map(|(name, entry)| name.string_size() + entry.strings_size())
            .sum::<u32>()
    }

    /// Returns the size of the descriptions in the tables children in the resource table.
    pub fn descriptions_size(&self) -> u32 {
        self.entries
            .iter()
            .map(|(_, entry)| entry.description_size())
            .sum::<u32>()
    }

    /// Returns the size of the data in in the tables children in the resource table.
    pub fn data_size(&self) -> u32 {
        self.entries
            .iter()
            .map(|(_, entry)| entry.data_size())
            .sum::<u32>()
    }
}

/// Raw resource data.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResourceData {
    data: DebugIgnore<Vec<u8>>,
    codepage: u32,
    reserved: u32,
}
impl Default for ResourceData {
    fn default() -> Self {
        Self {
            data: Vec::new().into(),
            codepage: CODE_PAGE_ID_EN_US as u32,
            reserved: 0,
        }
    }
}
impl ResourceData {
    /// Returns the raw data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the codepage of the data.
    pub fn codepage(&self) -> u32 {
        self.codepage
    }

    /// Set the raw data.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data.into();
    }

    /// Set the codepage of the data.
    pub fn set_codepage(&mut self, codepage: u32) {
        self.codepage = codepage;
    }
}

/// Resource entry in a resource table.
/// This can be either a child table or raw data.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResourceEntry {
    Table(ResourceTable),
    Data(ResourceData),
}
impl Default for ResourceEntry {
    fn default() -> Self {
        Self::Data(ResourceData::default())
    }
}
impl ResourceEntry {
    /// Returns if the data is a table.
    pub fn is_table(&self) -> bool {
        match self {
            ResourceEntry::Table(_) => true,
            ResourceEntry::Data(_) => false,
        }
    }

    /// Returns if the data is an entry.
    pub fn is_data(&self) -> bool {
        match self {
            ResourceEntry::Table(_) => false,
            ResourceEntry::Data(_) => true,
        }
    }

    /// Returns the sub-table if the data is an table.
    pub fn as_table(&self) -> Option<&ResourceTable> {
        match self {
            ResourceEntry::Table(table) => Some(table),
            ResourceEntry::Data(_) => None,
        }
    }

    /// Returns the mutable sub-table if the data is an table.
    pub fn as_table_mut(&mut self) -> Option<&mut ResourceTable> {
        match self {
            ResourceEntry::Table(table) => Some(table),
            ResourceEntry::Data(_) => None,
        }
    }

    /// Returns the table entry if the data is an entry.
    pub fn as_data(&self) -> Option<&ResourceData> {
        match self {
            ResourceEntry::Table(_) => None,
            ResourceEntry::Data(entry) => Some(entry),
        }
    }

    /// Returns the table entry if the data is an entry.
    pub fn as_data_mut(&mut self) -> Option<&mut ResourceData> {
        match self {
            ResourceEntry::Table(_) => None,
            ResourceEntry::Data(entry) => Some(entry),
        }
    }

    /// Returns the size of the table entry and its children in the resource table.
    pub fn table_size(&self) -> u32 {
        match self {
            // entry + sub-table
            ResourceEntry::Table(table) => table.tables_size() + 8,
            // entry
            ResourceEntry::Data(_) => 8,
        }
    }

    /// Returns the size of the strings in the entry and its children in the resource table.
    /// This is the size of the resouorce names of child tables.
    pub fn strings_size(&self) -> u32 {
        match self {
            ResourceEntry::Table(table) => table.strings_size(),
            ResourceEntry::Data(_) => 0,
        }
    }

    /// Returns the size of the descriptions in the entry and its children in the resource table.
    /// This is the size of the resource data description of the entry or child entries.
    pub fn description_size(&self) -> u32 {
        match self {
            ResourceEntry::Table(table) => table.descriptions_size(),
            ResourceEntry::Data(_) => 16,
        }
    }

    /// Returns the size of the data in the entry and its children in the resource table.
    /// This is the size of the resource data of the entry or child entries.
    pub fn data_size(&self) -> u32 {
        match self {
            ResourceEntry::Table(table) => table.data_size(),
            ResourceEntry::Data(data) => data.data.len() as u32,
        }
    }
}

/// Resource directory entry name.
/// This can either be a raw id or a name.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResourceEntryName {
    // raw id
    ID(u32),
    // 2 byte size + data
    Name(Vec<u8>),
}
impl Default for ResourceEntryName {
    fn default() -> Self {
        Self::ID(LANGUAGE_ID_EN_US as u32)
    }
}
impl ResourceEntryName {
    fn parse(image: &[u8], offset: u32, id: u32) -> Result<Self, ReadError> {
        if id & 0x80000000 != 0 {
            trace!("reading resource name {:#x?}", id);
            let address = offset + (id ^ 0x80000000);
            let length = read::<u16>(&image[address as usize..])? as u32;
            trace!("resource name length: {}", length);
            // size is in 16 bit characters so it needs to be doubled
            let data = &image[address as usize..(address + 2 + (length * 2)) as usize];
            trace!("resource name: {:x?}", data);
            Ok(Self::Name(data.to_vec()))
        } else {
            trace!("reading resource id {:#x?}", id);
            Ok(Self::ID(id))
        }
    }

    pub fn from_string<S: AsRef<str>>(string: S) -> Self {
        let string = string.as_ref();
        let mut data = Vec::with_capacity(string.len() * 2 + 2);
        data.extend_from_slice(&(string.len() as u16).to_le_bytes());
        data.extend(string.encode_utf16().flat_map(|c| c.to_le_bytes().to_vec()));
        Self::Name(data)
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Self::ID(_) => None,
            Self::Name(data) => {
                let length = read::<u16>(&data[0..]).unwrap() as usize;
                let data = &data[2..];
                let mut string = String::with_capacity(length);
                for i in 0..length {
                    let c = read::<u16>(&data[i * 2..]).unwrap() as u32;
                    string.push(core::char::from_u32(c).unwrap());
                }
                Some(string)
            }
        }
    }

    fn string_size(&self) -> u32 {
        match self {
            Self::ID(_) => 0,
            Self::Name(name) => name.len() as u32,
        }
    }

    fn id(&self) -> u32 {
        match self {
            Self::ID(id) => *id,
            Self::Name(_) => unreachable!(),
        }
    }

    fn string_data(&self) -> &[u8] {
        match self {
            Self::ID(_) => &[],
            Self::Name(data) => data.as_bytes(),
        }
    }
}

/// Version string table.
/// This is an entry in the version info resource.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct VersionStringTable {
    pub key: String,
    pub strings: IndexMap<String, String, RandomState>,
}

/// Version info resource.
/// This is a special resource that contains the version information of the executable.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct VersionInfo {
    pub info: FixedFileInfo,
    pub strings: Vec<VersionStringTable>,
    pub vars: Vec<VersionU16>,
}
impl VersionInfo {
    /// Parse the version info resource from a byte slice.
    ///
    /// # Returns
    /// Returns an error if the version info resource is not well-formed.
    pub fn parse(data: &[u8]) -> Result<Self, ReadError> {
        // read the version root header
        let header = read::<VersionHeader>(data)?;
        if header.length as usize > data.len() {
            return Err(ReadError(format!(
                "version root length {:#x?} is larger than data length {:#x?}",
                header.length,
                data.len()
            )));
        }

        let version_root_key =
            read_u16_string(&data[size_of::<VersionHeader>()..size_of::<VersionHeader>() + 32])?;
        if version_root_key != "VS_VERSION_INFO" {
            return Err(ReadError(format!(
                "invalid version root key: {:?}",
                version_root_key
            )));
        }
        let value_offset = aligned_to(
            size_of::<VersionHeader>() + version_root_key.len() * 2 + 2,
            4,
        );

        // read the fixed file info
        if header.value_length != size_of::<FixedFileInfo>() as u16 {
            return Err(ReadError(format!(
                "invalid file info length: {:#x?}",
                header.value_length
            )));
        }
        let info = read::<FixedFileInfo>(&data[value_offset..])?;
        if info.signature != 0xfeef04bd {
            return Err(ReadError(format!(
                "invalid fixed file info signature: {:#x?}",
                info.signature
            )));
        }
        let file_info_header_offset = aligned_to(value_offset + size_of::<FixedFileInfo>(), 4);

        let mut child_offset = file_info_header_offset;
        let mut strings = Vec::new();
        let mut vars = Vec::new();

        while child_offset < data.len() {
            // read the file info header
            let file_info_header = read::<VersionHeader>(&data[child_offset..])?;
            let file_info_end = child_offset + file_info_header.length as usize;
            let file_info_key =
                read_u16_string(&data[child_offset + size_of::<VersionHeader>()..file_info_end])?;
            let mut tables_offset = aligned_to(
                child_offset + size_of::<VersionHeader>() + file_info_key.len() * 2 + 2,
                4,
            );

            match file_info_key.as_str() {
                "VarFileInfo" => {
                    let var_header = read::<VersionHeader>(&data[tables_offset..])?;
                    let var_end = tables_offset + var_header.length as usize;
                    let table_key = read_u16_string(
                        &data[tables_offset + size_of::<VersionHeader>()..var_end],
                    )?;
                    if &table_key != "Translation" {
                        return Err(ReadError(format!("invalid var table key: {:?}", table_key)));
                    }
                    let vars_offset = aligned_to(
                        tables_offset + size_of::<VersionHeader>() + table_key.len() * 2 + 2,
                        4,
                    );
                    let mut var_offset = vars_offset;
                    while var_offset < var_end {
                        vars.push(read::<VersionU16>(&data[var_offset..])?);
                        var_offset += size_of::<u32>();
                    }
                }
                "StringFileInfo" => {
                    while tables_offset < child_offset + file_info_header.length as usize {
                        let string_table_header = read::<VersionHeader>(&data[tables_offset..])?;
                        let string_table_end = tables_offset + string_table_header.length as usize;
                        let string_table_key = read_u16_string(
                            &data[tables_offset + size_of::<VersionHeader>()..string_table_end],
                        )?;
                        let strings_offset = aligned_to(
                            tables_offset
                                + size_of::<VersionHeader>()
                                + string_table_key.len() * 2
                                + 2,
                            4,
                        );

                        let mut string_offset = strings_offset;
                        let mut string_table = VersionStringTable {
                            key: string_table_key,
                            strings: IndexMap::default(),
                        };

                        while string_offset < string_table_end {
                            let string_header = read::<VersionHeader>(&data[string_offset..])?;
                            string_offset += size_of::<VersionHeader>();
                            let string_key = read_u16_string(
                                &data[string_offset..string_offset + string_header.length as usize],
                            )?;
                            string_offset = aligned_to(string_offset + string_key.len() * 2 + 2, 4);

                            if string_header.value_length == 0 {
                                continue;
                            }
                            if string_header.type_ == 1 {
                                let string_value = read_u16_string(
                                    &data[string_offset
                                        ..string_offset + string_header.value_length as usize * 2],
                                )?;
                                string_table.strings.insert(string_key, string_value);
                            } else {
                                error!(
                                    "invalid string value type: {:#x?} (expected 0x1)",
                                    string_header.type_
                                );
                            };
                            string_offset = aligned_to(
                                string_offset + string_header.value_length as usize * 2,
                                4,
                            );
                        }
                        tables_offset = aligned_to(string_table_end, 4);
                        strings.push(string_table);
                    }
                }
                _ => {
                    return Err(ReadError(format!(
                        "invalid version string key: {:?}",
                        file_info_key
                    )));
                }
            }
            child_offset = aligned_to(child_offset + file_info_header.length as usize, 4);
        }

        Ok(Self {
            info,
            strings,
            vars,
        })
    }

    /// Build the version info into raw bytes to be included in a resource table.
    pub fn build(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let mut string_tables = Vec::new();
        for string_table_data in &self.strings {
            let mut string_table_children = Vec::new();
            for (key, value) in &string_table_data.strings {
                let mut string = Vec::new();
                string.extend(
                    VersionHeader {
                        length: ((aligned_to(6 + key.len() * 2 + 2, 4) + value.len() * 2 + 2)
                            as u16),
                        value_length: value.len() as u16 + 1,
                        type_: 1,
                    }
                    .as_bytes(),
                );
                string.extend(string_to_u16(key));
                string.extend(iter::repeat(0).take(aligned_to(string.len(), 4) - string.len()));
                string.extend(string_to_u16(value));
                string.extend(iter::repeat(0).take(aligned_to(string.len(), 4) - string.len()));
                string_table_children.extend(string);
            }
            let mut string_table = Vec::new();
            string_table.extend(
                VersionHeader {
                    length: (aligned_to(6 + string_table_data.key.len() * 2 + 2, 4)
                        + string_table_children.len()) as u16,
                    value_length: 0,
                    type_: 1,
                }
                .as_bytes(),
            );
            string_table.extend(string_to_u16(&string_table_data.key));
            string_table.extend(
                iter::repeat(0).take(aligned_to(string_table.len(), 4) - string_table.len()),
            );
            string_table.extend(string_table_children);
            string_tables.extend(string_table);
        }

        let mut string_info = Vec::new();
        string_info.extend(
            VersionHeader {
                length: (aligned_to(6 + "StringFileInfo".len() * 2 + 2, 4) + string_tables.len())
                    as u16,
                value_length: 0,
                type_: 1,
            }
            .as_bytes(),
        );
        string_info.extend(string_to_u16("StringFileInfo"));
        string_info
            .extend(iter::repeat(0).take(aligned_to(string_info.len(), 4) - string_info.len()));
        string_info.extend(string_tables);

        let mut var = Vec::new();
        var.extend(
            VersionHeader {
                length: (aligned_to(6 + "Translation".len() * 2 + 2, 4) + self.vars.len() * 4)
                    as u16,
                value_length: (self.vars.len() * 4) as u16,
                type_: 0,
            }
            .as_bytes(),
        );
        var.extend(string_to_u16("Translation"));
        var.extend(iter::repeat(0).take(aligned_to(var.len(), 4) - var.len()));
        var.extend(self.vars.iter().flat_map(|var| var.as_bytes()));
        var.extend(iter::repeat(0).take(aligned_to(var.len(), 4) - var.len()));

        let mut var_info = Vec::new();
        var_info.extend(
            VersionHeader {
                length: (aligned_to(6 + "VarFileInfo".len() * 2 + 2, 4) + var.len()) as u16,
                value_length: 0,
                type_: 1,
            }
            .as_bytes(),
        );
        var_info.extend(string_to_u16("VarFileInfo"));
        var_info.extend(iter::repeat(0).take(aligned_to(var_info.len(), 4) - var_info.len()));
        var_info.extend(var);

        data.extend(
            VersionHeader {
                length: (aligned_to(
                    aligned_to(6 + "VS_VERSION_INFO".len() * 2 + 2, 4) + size_of::<FixedFileInfo>(),
                    4,
                ) + string_info.len()
                    + var_info.len()) as u16,
                value_length: size_of::<FixedFileInfo>() as u16,
                type_: 0,
            }
            .as_bytes(),
        );
        data.extend(string_to_u16("VS_VERSION_INFO"));
        data.extend(iter::repeat(0).take(aligned_to(data.len(), 4) - data.len()));
        data.extend(self.info.as_bytes());
        data.extend(iter::repeat(0).take(aligned_to(data.len(), 4) - data.len()));
        data.extend(string_info);
        data.extend(var_info);

        data
    }
}
