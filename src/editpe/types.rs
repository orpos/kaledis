//! Portable executable data types.
//!
//! These types are a one-to-one mapping of the data described in <https://docs.microsoft.com/en-us/windows/win32/debug/pe-format>

use core::{mem, slice};

use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C, packed(1))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct VersionU8 {
    pub major: u8,
    pub minor: u8,
}
#[repr(C, packed(2))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct VersionU16 {
    pub major: u16,
    pub minor: u16,
}
#[repr(C, packed(4))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct VersionU32 {
    pub major: u32,
    pub minor: u32,
}
#[repr(C, packed(2))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct CoffHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}
#[repr(C, packed(2))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct StandardHeader {
    pub magic: u16,
    pub linker_version: VersionU8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, Default)]
pub struct WindowsHeader<UXX> {
    pub image_base: UXX,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub operating_system_version: VersionU16,
    pub image_version: VersionU16,
    pub subsystem_version: VersionU16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub check_sum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: UXX,
    pub size_of_stack_commit: UXX,
    pub size_of_heap_reserve: UXX,
    pub size_of_heap_commit: UXX,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
}
impl<UXX> WindowsHeader<UXX>
where
    UXX: IntoBytes,
{
    pub fn as_bytes(&self) -> &[u8] {
        // manually implement this here because zerocopy doesn't support derive for generic types
        unsafe {
            let len = mem::size_of_val(self);
            slice::from_raw_parts(self as *const Self as *const u8, len)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenericWindowsHeader {
    WindowsHeader32(WindowsHeader<u32>),
    WindowsHeader64(WindowsHeader<u64>),
}
impl GenericWindowsHeader {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.as_bytes(),
            GenericWindowsHeader::WindowsHeader64(header) => header.as_bytes(),
        }
    }

    pub const fn section_alignment(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.section_alignment,
            GenericWindowsHeader::WindowsHeader64(header) => header.section_alignment,
        }
    }

    pub const fn file_alignment(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.file_alignment,
            GenericWindowsHeader::WindowsHeader64(header) => header.file_alignment,
        }
    }

    pub const fn operating_system_version(&self) -> VersionU16 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.operating_system_version,
            GenericWindowsHeader::WindowsHeader64(header) => header.operating_system_version,
        }
    }

    pub const fn image_version(&self) -> VersionU16 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.image_version,
            GenericWindowsHeader::WindowsHeader64(header) => header.image_version,
        }
    }

    pub const fn subsystem_version(&self) -> VersionU16 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.subsystem_version,
            GenericWindowsHeader::WindowsHeader64(header) => header.subsystem_version,
        }
    }

    pub const fn win32_version_value(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.win32_version_value,
            GenericWindowsHeader::WindowsHeader64(header) => header.win32_version_value,
        }
    }

    pub const fn size_of_image(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.size_of_image,
            GenericWindowsHeader::WindowsHeader64(header) => header.size_of_image,
        }
    }

    pub const fn size_of_headers(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.size_of_headers,
            GenericWindowsHeader::WindowsHeader64(header) => header.size_of_headers,
        }
    }

    pub const fn check_sum(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.check_sum,
            GenericWindowsHeader::WindowsHeader64(header) => header.check_sum,
        }
    }

    pub const fn subsystem(&self) -> u16 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.subsystem,
            GenericWindowsHeader::WindowsHeader64(header) => header.subsystem,
        }
    }

    pub const fn dll_characteristics(&self) -> u16 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.dll_characteristics,
            GenericWindowsHeader::WindowsHeader64(header) => header.dll_characteristics,
        }
    }

    pub const fn loader_flags(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.loader_flags,
            GenericWindowsHeader::WindowsHeader64(header) => header.loader_flags,
        }
    }

    pub const fn number_of_rva_and_sizes(&self) -> u32 {
        match self {
            GenericWindowsHeader::WindowsHeader32(header) => header.number_of_rva_and_sizes,
            GenericWindowsHeader::WindowsHeader64(header) => header.number_of_rva_and_sizes,
        }
    }
}

#[repr(C, packed(4))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct ImageDataDirectory {
    pub virtual_address: u32,
    pub size: u32,
}

#[repr(C, packed(4))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct SectionHeader {
    pub name: u64,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}

impl SectionHeader {
    pub fn name(&self) -> Option<String> {
        let name = self.name.to_le_bytes();
        let name = core::str::from_utf8(
            &name[0..name.iter().position(|&c| c == b'\0').unwrap_or(name.len())],
        )
        .ok();
        name.map(|name| name.to_string())
    }
}

#[repr(C, packed(2))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct ResourceDirectoryTable {
    pub characteristics: u32,
    pub time_date_stamp: u32,
    pub version: VersionU16,
    pub number_of_name_entries: u16,
    pub number_of_id_entries: u16,
}

#[repr(C, packed(4))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct ResourceDirectoryEntry {
    pub name_offset_or_integer_id: u32,
    pub data_entry_or_subdirectory_offset: u32,
}

#[repr(C, packed(4))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct ResourceDataEntry {
    pub data_rva: u32,
    pub size: u32,
    pub codepage: u32,
    pub reserved: u32,
}

#[repr(C, packed(2))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct IconDirectory {
    pub reserved: u16,
    pub type_: u16,
    pub count: u16,
}

#[repr(C, packed(1))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct IconDirectoryEntry {
    pub width: u8,
    pub height: u8,
    pub color_count: u8,
    pub reserved: u8,
    pub planes: u16,
    pub bit_count: u16,
    pub bytes: u32,
    pub id: u16,
}

#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable)]
pub struct FixedFileInfo {
    pub signature: u32,
    pub struct_version: VersionU16,
    pub file_version: VersionU32,
    pub product_version: VersionU32,
    pub file_flags_mask: u32,
    pub file_flags: u32,
    pub file_os: u32,
    pub file_type: u32,
    pub file_subtype: u32,
    pub file_date: u64,
}
impl Default for FixedFileInfo {
    fn default() -> Self {
        Self {
            signature: 0xfeef04bd,
            struct_version: VersionU16 { major: 0, minor: 1 },
            file_version: VersionU32 { major: 1, minor: 0 },
            product_version: VersionU32 { major: 1, minor: 0 },
            file_flags_mask: 0x0000003f,
            file_flags: 0x00000000,
            file_os: 0x00040004,
            file_type: 0x00000001,
            file_subtype: 0x00000000,
            file_date: 0x00000000,
        }
    }
}

#[repr(C, packed(2))]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, FromBytes, IntoBytes, Immutable, Default,
)]
pub struct VersionHeader {
    pub length: u16,
    pub value_length: u16,
    pub type_: u16,
}
