use std::{fmt::format, fs::File, io::*, mem::size_of, path::Path, string};

use crate::{elf::*, node::SectionNode, parse::*};

pub fn gen_elf(input_filepath: &Path, output_filepath: &Path) -> File {
    let mut text = String::new();
    let mut input_file = File::open(input_filepath).expect("File not found");
    input_file
        .read_to_string(&mut text)
        .expect("Failed to read strings");

    let lines: Vec<&str> = text.split("\n").collect();
    let mut tokens = Vec::new();

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
    text_section_node.global_labels.push("_start".to_string());
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
                        push_global_labels(labels, section_node);
                    } else {
                        push_global_labels(labels, &mut text_section_node);
                    }
                }
                Directive::Section(section_name) => {
                    puah_current_label_with_instructions(
                        &mut current_label_with_instructions,
                        &mut current_section_node,
                        &mut text_section_node,
                    );

                    if !section_name.eq(".text") {
                        current_section_node = Some(SectionNode::new(section_name.clone()));
                    } else {
                        current_section_node = None;
                    }
                }
            },
            LineToken::Label(label) => {
                puah_current_label_with_instructions(
                    &mut current_label_with_instructions,
                    &mut current_section_node,
                    &mut text_section_node,
                );

                current_label_with_instructions = Some((label.clone(), Vec::new()));
            }
        }
    }

    puah_current_label_with_instructions(
        &mut current_label_with_instructions,
        &mut current_section_node,
        &mut text_section_node,
    );

    if current_section_node.is_some() {
        section_nodes.push(current_section_node.unwrap());
    }

    section_nodes.push(text_section_node);
    println!("{:#?}", section_nodes);

    let mut bytes: Vec<u8> = Vec::new();

    let mut header = Elf64Header::template();

    let mut symbol_table = vec![Elf64SymbolTableSection::default()];
    let mut string_table = vec![0x0];

    // file section
    symbol_table.push(Elf64SymbolTableSection::new(1, 4, 0, 65521, 0, 0));
    string_table.extend(format!("{}\0", input_filepath.to_str().unwrap()).as_bytes());

    let mut section_headers = vec![Elf64SectionHeader::default()];
    let mut section_header_string_table = vec![0x0];
    let mut data_bytes = Vec::new();

    for section_node in section_nodes.iter() {
        let mut data = Vec::new();

        for ins in section_node.default_instructions.iter() {
            data.extend(ins.mnemonic.get_opcode());
        }

        symbol_table.push(Elf64SymbolTableSection::new(
            0,
            3,
            0,
            (symbol_table.len() - 1) as u16,
            0,
            0,
        ));

        for (label, instructions) in section_node.labeled_instructions.iter() {
            for ins in instructions {
                data.extend(ins.mnemonic.get_opcode());
            }

            symbol_table.push(Elf64SymbolTableSection::new(
                string_table.len() as u32,
                2,
                0,
                (symbol_table.len() - 1) as u16,
                0,
                0,
            ));
            string_table.extend(format!("{}\0", label).as_bytes());
        }

        let data_len = data.len();

        section_headers.push(Elf64SectionHeader::new(
            section_header_string_table.len() as u32,
            1,
            6,
            0,
            0, // TODO
            data_len as u64,
            0,
            0,
            16,
            0,
        ));
        section_header_string_table.extend(format!("{}\0", section_node.name).as_bytes());

        // align 16bytes
        if data_len % 16 != 0 {
            for _ in 0..16 - (data_len % 16) {
                data.push(0x0);
            }
        }

        data_bytes.extend(data);
    }

    let mut shstrtab_section = Elf64SectionHeader::new(
        section_header_string_table.len() as u32,
        3,
        0,
        0,
        0, // TODO
        0, // TODO
        0,
        0,
        1,
        0,
    );
    section_header_string_table.extend(format!(".shstrtab\0").as_bytes());

    let symtab_section = Elf64SectionHeader::new(
        section_header_string_table.len() as u32,
        2,
        0,
        0,
        0, // TODO
        0, // TODO
        4,
        3,
        8,
        24,
    );
    section_header_string_table.extend(format!(".symtab\0").as_bytes());

    let strtab_section = Elf64SectionHeader::new(
        section_header_string_table.len() as u32,
        3,
        0,
        0,
        0, // TODO
        0, // TODO
        0,
        0,
        1,
        0,
    );
    section_header_string_table.extend(format!(".strtab\0").as_bytes());

    let section_header_string_table_len = section_header_string_table.len();
    shstrtab_section.set_size(section_header_string_table_len as u64);

    // align 16bytes
    if section_header_string_table_len % 16 != 0 {
        for _ in 0..16 - (section_header_string_table_len % 16) {
            section_header_string_table.push(0x0);
        }
    }

    section_headers.push(shstrtab_section);
    section_headers.push(symtab_section);
    section_headers.push(strtab_section);

    header.set_section_header_num(section_headers.len() as u16);
    header.set_section_header_str_index(3);

    bytes.extend(header.as_u8_slice());

    let mut _section_headers = Vec::<u8>::new();
    for section_header in section_headers.iter() {
        _section_headers.extend(section_header.as_u8_slice());
    }
    bytes.extend(_section_headers);

    bytes.extend(data_bytes);
    bytes.extend(section_header_string_table);

    let mut _symbol_table = Vec::<u8>::new();
    for symbol_table_section in symbol_table.iter() {
        _symbol_table.extend(symbol_table_section.as_u8_slice());
    }
    bytes.extend(_symbol_table);

    let string_table_len = string_table.len();
    // align 16bytes
    if string_table_len % 16 != 0 {
        for _ in 0..16 - (string_table_len % 16) {
            string_table.push(0x0);
        }
    }

    bytes.extend(string_table);

    // // extend instructions in text section
    // for section_node in section_nodes.iter() {
    //     if section_node.name.eq(".text") {
    //         // TODO: labeled instructions
    //         for ins in section_node.default_instructions.iter() {
    //             text.extend(ins.mnemonic.get_opcode());
    //         }
    //     }
    // }

    // // .text is 16byte align
    // let text_len = text.len();
    // let text_offset = size_of::<Elf64Header>() + (size_of::<Elf64SectionHeader>() * 5);
    // if text_len % 16 != 0 {
    //     for _ in 0..16 - (text_len % 16) {
    //         text.push(0x0);
    //     }
    // }

    // let mut section_header_string_table = vec![0x0];
    // section_header_string_table.extend(format!(".text\0").as_bytes());
    // section_header_string_table.extend(format!(".shstrtab\0").as_bytes());
    // section_header_string_table.extend(format!(".symtab\0").as_bytes());
    // section_header_string_table.extend(format!(".strtab\0").as_bytes());

    // let section_header_string_table_len = section_header_string_table.len();
    // let section_header_string_table_offset =
    //     header_bytes.len() + (size_of::<Elf64SectionHeader>() * 5) + text.len();

    // if section_header_string_table_len % 16 != 0 {
    //     for _ in 0..16 - (section_header_string_table_len % 16) {
    //         section_header_string_table.push(0x0);
    //     }
    // }

    // let null_section = Elf64SymbolTableSection::default();
    // // type: file, ./test/test.asm
    // let symbol_section_1 = Elf64SymbolTableSection::new(1, 4, 0, 65521, 0, 0);
    // // type: section, .text
    // let symbol_section_2 = Elf64SymbolTableSection::new(0, 3, 0, 1, 0, 0);
    // // type: notype, _start
    // let symbol_section_3 = Elf64SymbolTableSection::new(17, 16, 0, 1, 0, 0);

    // let symbol_table = [
    //     null_section.as_u8_slice(),
    //     symbol_section_1.as_u8_slice(),
    //     symbol_section_2.as_u8_slice(),
    //     symbol_section_3.as_u8_slice(),
    // ]
    // .concat();

    // let symbol_table_len = symbol_table.len();
    // let symbol_table_offset =
    //     section_header_string_table_offset + section_header_string_table.len();

    // // string table is 16byte align
    // let mut string_table = vec![0x0];
    // string_table.extend(format!("{}\0", input_filepath.to_str().unwrap()).as_bytes());
    // string_table.extend("_start\0".as_bytes());
    // let string_table_len = string_table.len();
    // let string_table_offset = symbol_table_offset + symbol_table_len;

    // if string_table_len % 16 != 0 {
    //     for _ in 0..16 - (string_table_len % 16) {
    //         string_table.push(0x0);
    //     }
    // }

    // let null_section = Elf64SectionHeader::default();
    // let text_section =
    //     Elf64SectionHeader::new(1, 1, 6, 0, text_offset as u64, text_len as u64, 0, 0, 16, 0);
    // let shstrtab_section = Elf64SectionHeader::new(
    //     7,
    //     3,
    //     0,
    //     0,
    //     section_header_string_table_offset as u64,
    //     section_header_string_table_len as u64,
    //     0,
    //     0,
    //     1,
    //     0,
    // );
    // let symtab_section = Elf64SectionHeader::new(
    //     17,
    //     2,
    //     0,
    //     0,
    //     symbol_table_offset as u64,
    //     symbol_table_len as u64,
    //     4,
    //     3,
    //     8,
    //     24,
    // );
    // let strtab_section = Elf64SectionHeader::new(
    //     25,
    //     3,
    //     0,
    //     0,
    //     string_table_offset as u64,
    //     string_table_len as u64,
    //     0,
    //     0,
    //     1,
    //     0,
    // );

    // let mut section_headers = vec![null_section];

    // section_headers.push(text_section);
    // section_headers.push(shstrtab_section);
    // section_headers.push(symtab_section);
    // section_headers.push(strtab_section);

    // let mut _section_headers = Vec::new();
    // for section_header in section_headers.iter() {
    //     _section_headers.extend(section_header.as_u8_slice());
    // }

    let mut file = File::create(output_filepath).expect("Failed to create file");

    // let bytes = [
    //     header_bytes,
    //     &_section_headers,
    //     &text,
    //     &section_header_string_table,
    //     &symbol_table,
    //     &string_table,
    // ]
    // .concat();
    file.write_all(&bytes).expect("Failed to write file");

    return file;
}

fn push_global_labels(labels: &Vec<String>, section_node: &mut SectionNode) {
    for label in labels {
        let mut is_found = false;
        for global_label in section_node.global_labels.iter() {
            if label.eq(global_label) {
                is_found = true;
                break;
            }
        }

        if !is_found {
            section_node.global_labels.push(label.clone());
        }
    }
}

fn push_labeled_instructions(
    label: &String,
    instructions: &Vec<Instruction>,
    section_node: &mut SectionNode,
) {
    if let Some((_, ins)) = section_node
        .labeled_instructions
        .iter_mut()
        .find(|(l, _)| label.eq(l))
    {
        ins.extend(instructions.clone());
    } else {
        section_node
            .labeled_instructions
            .push((label.clone(), instructions.clone()));
    }
}

fn puah_current_label_with_instructions(
    current_label_with_instructions: &mut Option<(String, Vec<Instruction>)>,
    current_section_node: &mut Option<SectionNode>,
    text_section_node: &mut SectionNode,
) {
    if let Some((ref current_label, ref instructions)) = current_label_with_instructions {
        if let Some(ref mut section_node) = current_section_node {
            push_labeled_instructions(current_label, instructions, section_node);
        } else {
            push_labeled_instructions(current_label, instructions, text_section_node);
        }
    }

    *current_label_with_instructions = None;
}
