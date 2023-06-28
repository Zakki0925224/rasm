use std::{fs::File, io::*, mem::size_of, path::Path};

use crate::{elf::*, parse::*};

pub fn gen_elf(input_filepath: &Path, output_filepath: &Path) -> File {
    let mut text = String::new();
    let mut input_file = File::open(input_filepath).expect("File not found");
    input_file
        .read_to_string(&mut text)
        .expect("Failed to read strings");

    let lines: Vec<&str> = text.split("\n").collect();
    let mut text = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let node = parse(*line);
        println!("line {}: \"{}\" => {:?}", i + 1, line, node);

        match node {
            LineNode::Instruction { opcode, operands } => text.extend(opcode.get_opcode()),
            _ => (),
        }
    }

    panic!("breakpoint");

    let header = Elf64Header::template();
    let header_bytes = header.as_u8_slice();

    // .text is 16byte align
    let text_len = text.len();
    let text_offset = size_of::<Elf64Header>() + (size_of::<Elf64SectionHeader>() * 5);
    if text_len % 16 != 0 {
        for _ in 0..16 - (text_len % 16) {
            text.push(0x0);
        }
    }

    let mut section_header_string_table = vec![
        0x0, 0x2e, 0x74, 0x65, 0x78, 0x74, 0x0, 0x2e, 0x73, 0x68, 0x73, 0x74, 0x72, 0x74, 0x61,
        0x62, 0x0, 0x2e, 0x73, 0x79, 0x6d, 0x74, 0x61, 0x62, 0x0, 0x2e, 0x73, 0x74, 0x72, 0x74,
        0x61, 0x62, 0x0,
    ];
    let section_header_string_table_len = section_header_string_table.len();
    let section_header_string_table_offset =
        header_bytes.len() + (size_of::<Elf64SectionHeader>() * 5) + text.len();

    if section_header_string_table_len % 16 != 0 {
        for _ in 0..16 - (section_header_string_table_len % 16) {
            section_header_string_table.push(0x0);
        }
    }

    let null_section = Elf64SymbolTableSection::default();
    let section_1 = Elf64SymbolTableSection::new(1, 4, 0, 65521, 0, 0);
    let section_2 = Elf64SymbolTableSection::new(0, 3, 0, 1, 0, 0);
    let section_3 = Elf64SymbolTableSection::new(17, 16, 0, 1, 0, 0);

    let symbol_table = [
        null_section.as_u8_slice(),
        section_1.as_u8_slice(),
        section_2.as_u8_slice(),
        section_3.as_u8_slice(),
    ]
    .concat();

    let symbol_table_len = symbol_table.len();
    let symbol_table_offset =
        section_header_string_table_offset + section_header_string_table.len();

    // string table is 16byte align
    let mut string_table = vec![0x0];
    string_table.extend(format!("{}\0", input_filepath.to_str().unwrap()).as_bytes());
    string_table.extend("_start\0".as_bytes());
    let string_table_len = string_table.len();
    let string_table_offset = symbol_table_offset + symbol_table_len;

    if string_table_len % 16 != 0 {
        for _ in 0..16 - (string_table_len % 16) {
            string_table.push(0x0);
        }
    }

    let null_section = Elf64SectionHeader::default();
    let text_section =
        Elf64SectionHeader::new(1, 1, 6, 0, text_offset as u64, text_len as u64, 0, 0, 16, 0);
    let shstrtab_section = Elf64SectionHeader::new(
        7,
        3,
        0,
        0,
        section_header_string_table_offset as u64,
        section_header_string_table_len as u64,
        0,
        0,
        1,
        0,
    );
    let symtab_section = Elf64SectionHeader::new(
        17,
        2,
        0,
        0,
        symbol_table_offset as u64,
        symbol_table_len as u64,
        4,
        3,
        8,
        24,
    );
    let strtab_section = Elf64SectionHeader::new(
        25,
        3,
        0,
        0,
        string_table_offset as u64,
        string_table_len as u64,
        0,
        0,
        1,
        0,
    );

    let mut file = File::create(output_filepath).expect("Failed to create file");

    let bytes = [
        header_bytes,
        null_section.as_u8_slice(),
        text_section.as_u8_slice(),
        shstrtab_section.as_u8_slice(),
        symtab_section.as_u8_slice(),
        strtab_section.as_u8_slice(),
        &text,
        &section_header_string_table,
        &symbol_table,
        &string_table,
    ]
    .concat();
    file.write_all(&bytes).expect("Failed to write file");

    return file;
}
