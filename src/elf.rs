use byteorder::{ByteOrder, LittleEndian};
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
pub struct Elf64SectionHeader {
    name: [u8; 4],
    s_type: [u8; 4],
    flags: [u8; 8],
    addr: [u8; 8],
    offset: [u8; 8],
    size: [u8; 8],
    link: [u8; 4],
    info: [u8; 4],
    align: [u8; 8],
    entry_size: [u8; 8],
}

impl Elf64SectionHeader {
    pub fn new(
        name: u32,
        s_type: u32,
        flags: u64,
        addr: u64,
        offset: u64,
        size: u64,
        link: u32,
        info: u32,
        align: u64,
        entry_size: u64,
    ) -> Self {
        let mut header = Self::default();
        header.set_name(name);
        header.set_s_type(s_type);
        header.set_flags(flags);
        header.set_addr(addr);
        header.set_offset(offset);
        header.set_size(size);
        header.set_link(link);
        header.set_info(info);
        header.set_align(align);
        header.set_entry_size(entry_size);

        return header;
    }

    pub fn as_u8_slice(&self) -> &[u8] {
        return unsafe { from_raw_parts((self as *const Self) as *const u8, size_of::<Self>()) };
    }

    pub fn name(&self) -> u32 {
        return LittleEndian::read_u32(&self.name);
    }

    pub fn set_name(&mut self, name: u32) {
        let mut name_buf = [0; 4];
        LittleEndian::write_u32(&mut name_buf, name);
        self.name = name_buf;
    }

    pub fn s_type(&self) -> u32 {
        return LittleEndian::read_u32(&self.s_type);
    }

    pub fn set_s_type(&mut self, s_type: u32) {
        let mut s_type_buf = [0; 4];
        LittleEndian::write_u32(&mut s_type_buf, s_type);
        self.s_type = s_type_buf;
    }

    pub fn flags(&self) -> u64 {
        return LittleEndian::read_u64(&self.flags);
    }

    pub fn set_flags(&mut self, flags: u64) {
        let mut flags_buf = [0; 8];
        LittleEndian::write_u64(&mut flags_buf, flags);
        self.flags = flags_buf;
    }

    pub fn addr(&self) -> u64 {
        return LittleEndian::read_u64(&self.addr);
    }

    pub fn set_addr(&mut self, addr: u64) {
        let mut addr_buf = [0; 8];
        LittleEndian::write_u64(&mut addr_buf, addr);
        self.addr = addr_buf;
    }

    pub fn offset(&self) -> u64 {
        return LittleEndian::read_u64(&self.offset);
    }

    pub fn set_offset(&mut self, offset: u64) {
        let mut offset_buf = [0; 8];
        LittleEndian::write_u64(&mut offset_buf, offset);
        self.offset = offset_buf;
    }

    pub fn size(&self) -> u64 {
        return LittleEndian::read_u64(&self.size);
    }

    pub fn set_size(&mut self, size: u64) {
        let mut size_buf = [0; 8];
        LittleEndian::write_u64(&mut size_buf, size);
        self.size = size_buf;
    }

    pub fn link(&self) -> u32 {
        return LittleEndian::read_u32(&self.link);
    }

    pub fn set_link(&mut self, link: u32) {
        let mut link_buf = [0; 4];
        LittleEndian::write_u32(&mut link_buf, link);
        self.link = link_buf;
    }

    pub fn info(&self) -> u32 {
        return LittleEndian::read_u32(&self.info);
    }

    pub fn set_info(&mut self, info: u32) {
        let mut info_buf = [0; 4];
        LittleEndian::write_u32(&mut info_buf, info);
        self.info = info_buf;
    }

    pub fn align(&self) -> u64 {
        return LittleEndian::read_u64(&self.align);
    }

    pub fn set_align(&mut self, align: u64) {
        let mut align_buf = [0; 8];
        LittleEndian::write_u64(&mut align_buf, align);
        self.align = align_buf;
    }

    pub fn entry_size(&self) -> u64 {
        return LittleEndian::read_u64(&self.entry_size);
    }

    pub fn set_entry_size(&mut self, entry_size: u64) {
        let mut entry_size_buf = [0; 8];
        LittleEndian::write_u64(&mut entry_size_buf, entry_size);
        self.entry_size = entry_size_buf;
    }
}

impl Default for Elf64SectionHeader {
    fn default() -> Self {
        Self {
            name: [0; 4],
            s_type: [0; 4],
            flags: [0; 8],
            addr: [0; 8],
            offset: [0; 8],
            size: [0; 8],
            link: [0; 4],
            info: [0; 4],
            align: [0; 8],
            entry_size: [0; 8],
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Elf64SymbolTableSection {
    name: [u8; 4],
    info: u8,
    other: u8,
    index: [u8; 2],
    value: [u8; 8],
    size: [u8; 8],
}

impl Elf64SymbolTableSection {
    pub fn new(name: u32, info: u8, other: u8, index: u16, value: u64, size: u64) -> Self {
        let mut section = Self::default();
        section.set_name(name);
        section.set_info(info);
        section.set_other(other);
        section.set_index(index);
        section.set_value(value);
        section.set_size(size);

        return section;
    }

    pub fn as_u8_slice(&self) -> &[u8] {
        return unsafe { from_raw_parts((self as *const Self) as *const u8, size_of::<Self>()) };
    }

    pub fn name(&self) -> u32 {
        return LittleEndian::read_u32(&self.name);
    }

    pub fn set_name(&mut self, name: u32) {
        let mut name_buf = [0; 4];
        LittleEndian::write_u32(&mut name_buf, name);
        self.name = name_buf;
    }

    pub fn info(&self) -> u8 {
        return self.info;
    }

    pub fn set_info(&mut self, info: u8) {
        self.info = info;
    }

    pub fn other(&self) -> u8 {
        return self.other;
    }

    pub fn set_other(&mut self, other: u8) {
        self.other = other;
    }

    pub fn index(&self) -> u16 {
        return LittleEndian::read_u16(&self.index);
    }

    pub fn set_index(&mut self, index: u16) {
        let mut index_buf = [0; 2];
        LittleEndian::write_u16(&mut index_buf, index);
        self.index = index_buf;
    }

    pub fn value(&self) -> u64 {
        return LittleEndian::read_u64(&self.value);
    }

    pub fn set_value(&mut self, value: u64) {
        let mut value_buf = [0; 8];
        LittleEndian::write_u64(&mut value_buf, value);
        self.value = value_buf;
    }

    pub fn size(&self) -> u64 {
        return LittleEndian::read_u64(&self.size);
    }

    pub fn set_size(&mut self, size: u64) {
        let mut size_buf = [0; 8];
        LittleEndian::write_u64(&mut size_buf, size);
        self.size = size_buf;
    }
}

impl Default for Elf64SymbolTableSection {
    fn default() -> Self {
        return Self {
            name: [0; 4],
            info: 0,
            other: 0,
            index: [0; 2],
            value: [0; 8],
            size: [0; 8],
        };
    }
}
