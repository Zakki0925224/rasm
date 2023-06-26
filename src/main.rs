use std::{
    env,
    fs::File,
    io::{Stdout, Write},
    path::Path,
    process::Command,
};

use crate::{generator::gen_elf, parse::parse};

mod elf;
mod generator;
mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Invalid arguments");
    }

    let input_filepath = Path::new(&args[1]);
    let _buf = input_filepath.with_extension("o");
    let output_filepath = _buf.as_path();
    gen_elf(input_filepath, output_filepath);

    // link
    // let out = Command::new("ld")
    //     .args([
    //         "-m",
    //         "elf_x86_64",
    //         "-o",
    //         input_filepath
    //             .clone()
    //             .with_extension("elf")
    //             .to_str()
    //             .unwrap(),
    //         output_filepath.to_str().unwrap(),
    //     ])
    //     .output()
    //     .expect("Failed to execute ld");
    //println!("{:?}", out);
}

#[test]
fn test() {
    let asm = "
        ; this is a comment
        nop
        syscall
        ";

    // rasm binary
    let input_filepath = Path::new("./test/test.asm");
    let _buf = input_filepath.with_extension("o");
    let output_filepath = _buf.as_path();
    let mut file = File::create(input_filepath).unwrap();
    file.write(asm.as_bytes()).unwrap();

    gen_elf(input_filepath, output_filepath);

    // nasm binary
    let _buf = input_filepath.with_extension("nasmo");
    let nasm_output_filepath = _buf.as_path();

    let out = Command::new("nasm")
        .args([
            "-f",
            "elf64",
            input_filepath.to_str().unwrap(),
            "-o",
            nasm_output_filepath.to_str().unwrap(),
        ])
        .output();

    assert_eq!(out.is_ok(), true);

    let out = Command::new("cmp")
        .args([
            nasm_output_filepath.to_str().unwrap(),
            output_filepath.to_str().unwrap(),
        ])
        .output();

    match out {
        Ok(output) => assert!(
            output.stdout.len() == 0,
            "{}",
            String::from_utf8(output.stdout).unwrap()
        ),
        Err(err) => panic!("{:?}", err),
    };
}
