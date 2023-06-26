use std::{mem::size_of, slice::from_raw_parts};

pub const MAGIC_NUMS: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

#[derive(Debug)]
#[repr(C)]
pub struct Elf64Header {
    pub magic_nums: [u8; 4],
    pub class: u8,
    pub endian: u8,
    pub version: u8,
    pub abi: u8,
    pub abi_version: u8,
    reserved: [u8; 7],
    pub object_type: [u8; 2],
    pub machine_type: [u8; 2],
    pub version2: [u8; 4],
    pub entry: [u8; 8],
    pub program_header_offset: [u8; 8],
    pub section_header_offset: [u8; 8],
    pub flags: [u8; 4],
    pub header_size: [u8; 2],
    pub program_header_size: [u8; 2],
    pub program_header_num: [u8; 2],
    pub section_header_size: [u8; 2],
    pub section_header_num: [u8; 2],
    pub section_header_str_index: [u8; 2],
}

impl Elf64Header {
    pub fn template() -> Self {
        return Self {
            magic_nums: MAGIC_NUMS,
            class: 0x2,
            endian: 0x1,
            version: 0x1,
            abi: 0x0,
            abi_version: 0x0,
            reserved: [0x0; 7],
            object_type: [0x1, 0x0],
            machine_type: [0x3e, 0x0],
            version2: [0x1, 0x0, 0x0, 0x0],
            entry: [0x0; 8],
            program_header_offset: [0x0; 8],
            section_header_offset: [0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0],
            flags: [0x0; 4],
            header_size: [0x40, 0x0],
            program_header_size: [0x0, 0x0],
            program_header_num: [0x0, 0x0],
            section_header_size: [0x40, 0x0],
            section_header_num: [0x5, 0x0],
            section_header_str_index: [0x2, 0x0],
        };
    }

    pub fn as_u8_slice(&self) -> &[u8] {
        return unsafe { from_raw_parts((self as *const Self) as *const u8, size_of::<Self>()) };
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Elf64ProgramHeader {
    pub h_type: [u8; 4],
    pub flags: [u8; 4],
    pub offset: [u8; 8],
    pub virt_addr: [u8; 8],
    pub phys_addr: [u8; 8],
    pub file_size: [u8; 8],
    pub mem_size: [u8; 8],
    pub align: [u8; 8],
}

#[derive(Debug)]
#[repr(C)]
pub struct Elf64SectionHeader {
    pub name: [u8; 4],
    pub s_type: [u8; 4],
    pub flags: [u8; 4],
    pub addr: [u8; 8],
    pub offset: [u8; 8],
    pub size: [u8; 8],
    pub link: [u8; 4],
    pub info: [u8; 4],
    pub align: [u8; 8],
    pub entry_size: [u8; 8],
}
