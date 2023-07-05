use std::{fs::File, io::*, mem::size_of, path::Path};

use crate::{elf::*, node::SectionNode, parse::*};

pub fn gen_elf(input_filepath: &Path, output_filepath: &Path) -> File {
    let mut text = String::new();
    let mut input_file = File::open(input_filepath).expect("File not found");
    input_file
        .read_to_string(&mut text)
        .expect("Failed to read strings");

    let lines: Vec<&str> = text.split("\n").collect();
    let mut tokens = Vec::new();
    let mut text = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let token = parse(*line);
        println!("line {}: \"{}\" => {:?}", i + 1, line, token);
        tokens.push(token);
    }

    match check_tokens(&tokens) {
        CheckResult::Ok => (),
        CheckResult::Error { at, error_type } => {
            println!(
                "{}, line{}: \"{}\" is {:?}",
                input_filepath.to_str().unwrap(),
                at + 1,
                lines[at],
                error_type
            );
            panic!("Parse error");
        }
    }

    let mut section_nodes = Vec::new();
    let mut text_section_node = SectionNode::new(".text".to_string());
    let mut current_section_node: Option<SectionNode> = None;
    let mut current_label_with_instructions: Option<(String, Vec<Instruction>)> = None;

    for (_, token) in tokens.iter().enumerate() {
        match token {
            LineToken::Invalid => unreachable!(), // have to paniced at token checker
            LineToken::Empty => continue,
            LineToken::Comment => continue,
            LineToken::Instruction(ins) => {
                if let Some((_, ref mut instructions)) = current_label_with_instructions {
                    instructions.push(ins.clone());
                } else {
                    if let Some(ref mut section_node) = current_section_node {
                        section_node.default_instructions.push(ins.clone());
                    } else {
                        text_section_node.default_instructions.push(ins.clone());
                    }
                }
            }
            LineToken::Directive(dir) => match dir {
                Directive::Global(labels) => {
                    if let Some(ref mut section_node) = current_section_node {
                        section_node.global_labels.extend(labels.clone());
                    } else {
                        text_section_node.global_labels.extend(labels.clone());
                    }
                }
                Directive::Section(section_name) => {
                    if let Some(ref section_node) = current_section_node {
                        section_nodes.push(section_node.clone());
                    }

                    if !section_name.eq(".text") {
                        current_section_node = Some(SectionNode::new(section_name.clone()));
                    }
                }
            },
            LineToken::Label(label) => {
                if let Some((ref current_label, ref instructions)) = current_label_with_instructions
                {
                    if !label.eq(current_label) {
                        if let Some(ref mut section_node) = current_section_node {
                            section_node
                                .labeled_instructions
                                .push((current_label.clone(), instructions.clone()));
                        } else {
                            text_section_node
                                .labeled_instructions
                                .push((current_label.clone(), instructions.clone()));
                        }
                    }
                }

                current_label_with_instructions = Some((label.clone(), Vec::new()));
            }
        }
    }

    if current_label_with_instructions.is_some() {
        if let Some(ref mut section_node) = current_section_node {
            section_node
                .labeled_instructions
                .push(current_label_with_instructions.unwrap());
        } else {
            text_section_node
                .labeled_instructions
                .push(current_label_with_instructions.unwrap());
        }
    }

    if current_section_node.is_some() {
        section_nodes.push(current_section_node.unwrap());
    }

    section_nodes.push(text_section_node);
    println!("{:?}", section_nodes);

    let header = Elf64Header::template();
    let header_bytes = header.as_u8_slice();

    // extend instructions in text section
    for section_node in section_nodes.iter() {
        if section_node.name.eq(".text") {
            // TODO: labeled instructions
            for ins in section_node.default_instructions.iter() {
                text.extend(ins.opcode.get_opcode());
            }
        }
    }

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
