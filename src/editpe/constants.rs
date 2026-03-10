//! Windows API and binary constants.

#![allow(non_upper_case_globals)]

pub type DWORD = u32;
pub type UINT = u32;
pub type WORD = u16;
pub type LANGID = WORD;


pub const VS_COMMENTS: &str = "Comments";
pub const VS_COMPANY_NAME: &str = "CompanyName";
pub const VS_FILE_DESCRIPTION: &str = "FileDescription";
pub const VS_FILE_VERSION: &str = "FileVersion";
pub const VS_INTERNAL_NAME: &str = "InternalName";
pub const VS_LEGAL_COPYRIGHT: &str = "LegalCopyright";
pub const VS_LEGAL_TRADEMARKS: &str = "LegalTrademarks";
pub const VS_ORIGINAL_FILENAME: &str = "OriginalFilename";
pub const VS_PRIVATE_BUILD: &str = "PrivateBuild";
pub const VS_PRODUCT_NAME: &str = "ProductName";
pub const VS_PRODUCT_VERSION: &str = "ProductVersion";
pub const VS_SPECIAL_BUILD: &str = "SpecialBuild";


// https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-lcid/a9eac961-e77d-41a6-90a5-ce1a8b0cdb9c
pub const LANGUAGE_ID_EN_US: LANGID = 1033; // 0x0409, en-US
// https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-ucoderef/28fefe92-d66c-4b03-90a9-97b473223d43
pub const CODE_PAGE_ID_EN_US: LANGID = 1200; // 0x04B0, UTF-16LE


// https://docs.microsoft.com/en-us/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo

pub const VOS_UNKNOWN: DWORD = 0x00000000;
pub const VOS_DOS: DWORD = 0x00010000;
pub const VOS_NT: DWORD = 0x00040000;
pub const VOS__WINDOWS16: DWORD = 0x00000001;
pub const VOS__WINDOWS32: DWORD = 0x00000004;
pub const VOS_OS216: DWORD = 0x00020000;
pub const VOS_OS232: DWORD = 0x00030000;
pub const VOS__PM16: DWORD = 0x00000002;
pub const VOS__PM32: DWORD = 0x00000003;

pub const VFT_UNKNOWN: DWORD = 0x00000000;
pub const VFT_APP: DWORD = 0x00000001;
pub const VFT_DLL: DWORD = 0x00000002;
pub const VFT_DRV: DWORD = 0x00000003;
pub const VFT_FONT: DWORD = 0x00000004;
pub const VFT_STATIC_LIB: DWORD = 0x00000007;
pub const VFT_VXD: DWORD = 0x00000005;

pub const VFT2_UNKNOWN: DWORD = 0x00000000;
pub const VFT2_DRV_COMM: DWORD = 0x0000000A;
pub const VFT2_DRV_DISPLAY: DWORD = 0x00000004;
pub const VFT2_DRV_INSTALLABLE: DWORD = 0x00000008;
pub const VFT2_DRV_KEYBOARD: DWORD = 0x00000002;
pub const VFT2_DRV_LANGUAGE: DWORD = 0x00000003;
pub const VFT2_DRV_MOUSE: DWORD = 0x00000005;
pub const VFT2_DRV_NETWORK: DWORD = 0x00000006;
pub const VFT2_DRV_PRINTER: DWORD = 0x00000001;
pub const VFT2_DRV_SOUND: DWORD = 0x00000009;
pub const VFT2_DRV_SYSTEM: DWORD = 0x00000007;
pub const VFT2_DRV_VERSIONED_PRINTER: DWORD = 0x0000000C;
pub const VFT2_FONT_RASTER: DWORD = 0x00000001;
pub const VFT2_FONT_TRUETYPE: DWORD = 0x00000003;
pub const VFT2_FONT_VECTOR: DWORD = 0x00000002;

pub const VS_FIXEDFILEINFO_SIGNATURE: DWORD = 0xFEEF04BD;
pub const VS_FIXEDFILEINFO_VERSION: DWORD = 0x00010000;


// https://docs.microsoft.com/en-us/windows/win32/debug/pe-format

pub const PE_DOS_MAGIC: WORD = 0x5a4d; // MZ
pub const PE_PTR_OFFSET: DWORD = 0x03c;
pub const PE_NT_SIGNATURE: DWORD = 0x00004550; // PE00
pub const PE_32_MAGIC: WORD = 0x010b;
pub const PE_64_MAGIC: WORD = 0x020b;


// https://docs.microsoft.com/en-us/windows/win32/menurc/resource-types

pub const RT_CURSOR: WORD = 0x01;
pub const RT_BITMAP: WORD = 0x02;
pub const RT_ICON: WORD = 0x03;
pub const RT_MENU: WORD = 0x04;
pub const RT_DIALOG: WORD = 0x05;
pub const RT_STRING: WORD = 0x06;
pub const RT_FONTDIR: WORD = 0x07;
pub const RT_FONT: WORD = 0x08;
pub const RT_ACCELERATOR: WORD = 0x09;
pub const RT_RCDATA: WORD = 0x0A;
pub const RT_MESSAGETABLE: WORD = 0x0B;
pub const RT_GROUP_CURSOR: WORD = 0x0C;
pub const RT_GROUP_ICON: WORD = 0x0E;
pub const RT_VERSION: WORD = 0x10;
pub const RT_DLGINCLUDE: WORD = 0x11;
pub const RT_PLUGPLAY: WORD = 0x13;
pub const RT_VXD: WORD = 0x14;
pub const RT_ANICURSOR: WORD = 0x15;
pub const RT_ANIICON: WORD = 0x16;
pub const RT_HTML: WORD = 0x17;
pub const RT_MANIFEST: WORD = 0x18;


// https://docs.microsoft.com/en-us/windows/win32/debug/pe-format#section-flags

pub const IMAGE_SCN_TYPE_NO_PAD: DWORD = 0x00000008;
pub const IMAGE_SCN_CNT_CODE: DWORD = 0x00000020;
pub const IMAGE_SCN_CNT_INITIALIZED_DATA: DWORD = 0x00000040;
pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: DWORD = 0x00000080;
pub const IMAGE_SCN_LNK_OTHER: DWORD = 0x00000100;
pub const IMAGE_SCN_LNK_INFO: DWORD = 0x00000200;
pub const IMAGE_SCN_LNK_REMOVE: DWORD = 0x00000800;
pub const IMAGE_SCN_LNK_COMDAT: DWORD = 0x00001000;
pub const IMAGE_SCN_GPREL: DWORD = 0x00008000;
pub const IMAGE_SCN_MEM_PURGEABLE: DWORD = 0x00020000;
pub const IMAGE_SCN_MEM_16BIT: DWORD = 0x00020000;
pub const IMAGE_SCN_MEM_LOCKED: DWORD = 0x00040000;
pub const IMAGE_SCN_MEM_PRELOAD: DWORD = 0x00080000;
pub const IMAGE_SCN_ALIGN_1BYTES: DWORD = 0x00100000;
pub const IMAGE_SCN_ALIGN_2BYTES: DWORD = 0x00200000;
pub const IMAGE_SCN_ALIGN_4BYTES: DWORD = 0x00300000;
pub const IMAGE_SCN_ALIGN_8BYTES: DWORD = 0x00400000;
pub const IMAGE_SCN_ALIGN_16BYTES: DWORD = 0x00500000;
pub const IMAGE_SCN_ALIGN_32BYTES: DWORD = 0x00600000;
pub const IMAGE_SCN_ALIGN_64BYTES: DWORD = 0x00700000;
pub const IMAGE_SCN_ALIGN_128BYTES: DWORD = 0x00800000;
pub const IMAGE_SCN_ALIGN_256BYTES: DWORD = 0x00900000;
pub const IMAGE_SCN_ALIGN_512BYTES: DWORD = 0x00A00000;
pub const IMAGE_SCN_ALIGN_1024BYTES: DWORD = 0x00B00000;
pub const IMAGE_SCN_ALIGN_2048BYTES: DWORD = 0x00C00000;
pub const IMAGE_SCN_ALIGN_4096BYTES: DWORD = 0x00D00000;
pub const IMAGE_SCN_ALIGN_8192BYTES: DWORD = 0x00E00000;
pub const IMAGE_SCN_LNK_NRELOC_OVFL: DWORD = 0x01000000;
pub const IMAGE_SCN_MEM_DISCARDABLE: DWORD = 0x02000000;
pub const IMAGE_SCN_MEM_NOT_CACHED: DWORD = 0x04000000;
pub const IMAGE_SCN_MEM_NOT_PAGED: DWORD = 0x08000000;
pub const IMAGE_SCN_MEM_SHARED: DWORD = 0x10000000;
pub const IMAGE_SCN_MEM_EXECUTE: DWORD = 0x20000000;
pub const IMAGE_SCN_MEM_READ: DWORD = 0x40000000;
pub const IMAGE_SCN_MEM_WRITE: DWORD = 0x80000000;

// https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#windows-subsystem

pub const IMAGE_SUBSYSTEM_UNKNOWN: WORD = 0;
pub const IMAGE_SUBSYSTEM_NATIVE: WORD = 1;
pub const IMAGE_SUBSYSTEM_WINDOWS_GUI: WORD = 2;
pub const IMAGE_SUBSYSTEM_WINDOWS_CUI: WORD = 3;
pub const IMAGE_SUBSYSTEM_OS2_CUI: WORD = 5;
pub const IMAGE_SUBSYSTEM_POSIX_CUI: WORD = 7;
pub const IMAGE_SUBSYSTEM_NATIVE_WINDOWS: WORD = 8;
pub const IMAGE_SUBSYSTEM_WINDOWS_CE_GUI: WORD = 9;
pub const IMAGE_SUBSYSTEM_EFI_APPLICATION: WORD = 10;
pub const IMAGE_SUBSYSTEM_EFI_BOOT_SERVICE_DRIVER: WORD = 11;
pub const IMAGE_SUBSYSTEM_EFI_RUNTIME_DRIVER: WORD = 12;
pub const IMAGE_SUBSYSTEM_EFI_ROM: WORD = 13;
pub const IMAGE_SUBSYSTEM_XBOX: WORD = 14;
pub const IMAGE_SUBSYSTEM_WINDOWS_BOOT_APPLICATION: WORD = 16;
