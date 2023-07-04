use std::{fs::File, io::*, mem::size_of, path::Path};

use crate::{elf::*, parse::*};

pub fn gen_elf(input_filepath: &Path, output_filepath: &Path) -> File {
    let mut text = String::new();
    let mut input_file = File::open(input_filepath).expect("File not found");
    input_file
        .read_to_string(&mut text)
        .expect("Failed to read strings");

    let lines: Vec<&str> = text.split("\n").collect();
    let mut nodes = Vec::new();
    let mut text = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let node = parse(*line);
        println!("line {}: \"{}\" => {:?}", i + 1, line, node);
        nodes.push(node);
    }

    match check_nodes(&nodes) {
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

    let mut global_labels = Vec::new();

    let mut section_with_label_with_instructions = Vec::new();
    let mut current_section = None;

    let mut label_with_instructions = Vec::new();

    let mut current_label = None;
    let mut nodes_in_current_label = Vec::new();

    for node in nodes {
        match node {
            LineNode::Directive(directive) => match directive {
                Directive::Global(targets) => global_labels.extend(targets),
                Directive::Section(section_name) => {
                    label_with_instructions
                        .push((current_label.clone(), nodes_in_current_label.to_vec()));
                    nodes_in_current_label.clear();

                    section_with_label_with_instructions
                        .push((current_section, label_with_instructions.to_vec()));
                    label_with_instructions.clear();

                    current_section = Some(section_name);
                }
            },
            LineNode::Label(label) => {
                if nodes_in_current_label.len() > 0 {
                    label_with_instructions.push((current_label, nodes_in_current_label.to_vec()));
                    nodes_in_current_label.clear();
                }

                current_label = Some(label);
            }
            LineNode::Invalid => unreachable!(),
            LineNode::Empty | LineNode::Comment => (),
            node => {
                nodes_in_current_label.push(node);
            }
        }
    }

    if nodes_in_current_label.len() > 0 {
        label_with_instructions.push((current_label, nodes_in_current_label.to_vec()));
    }

    if label_with_instructions.len() > 0 {
        section_with_label_with_instructions
            .push((current_section, label_with_instructions.to_vec()));
    }

    println!("{:?}", section_with_label_with_instructions);

    for global_label in global_labels.iter() {
        let mut found = false;

        'outer: for (_, label_with_instructions) in section_with_label_with_instructions.iter() {
            for (label, _) in label_with_instructions {
                if let Some(label) = label {
                    if label.eq(global_label) {
                        found = true;
                        break 'outer;
                    }
                }
            }
        }

        if !found {
            panic!("Global label \"{}\" was not defined", global_label);
        }
    }

    for (section, label_with_instructions) in section_with_label_with_instructions.iter() {
        if let Some(section) = section {
            if section != ".text" {
                panic!("Unsupported section");
            }
        } else {
            continue;
        }

        for (label, instructions) in label_with_instructions.iter() {
            if let Some(label) = label {
                if label.eq("_start") {
                    for node in instructions {
                        match node {
                            LineNode::Instruction { opcode, operands } => {
                                text.extend(opcode.get_opcode());
                            }
                            _ => (),
                        }
                    }
                } else {
                    panic!("Unsupported label");
                }
            }
        }
    }

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
