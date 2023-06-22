// opcodes
const OP_SYSCALL: [u8; 2] = [0x0f, 0x05];
const OP_MOV_RM32_IMM32: [u8; 2] = [0x48, 0xc7];

#[derive(Debug)]
pub enum Opcode {
    Syscall,
    MovRm32Imm32, // copy imm32 to rm32
}

impl Opcode {
    pub fn get_opcode(&self) -> Vec<u8> {
        return match self {
            Opcode::Syscall => OP_SYSCALL.to_vec(),
            _ => unreachable!(),
        };
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub opcode: Option<Opcode>,
    pub operands: Vec<u8>,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidInstruction(String),
}

pub fn parse(line: &str) -> Result<Instruction, ParseError> {
    let mut opcode = None;

    let mut chars_vec = Vec::new();
    let mut collecting_chars = false;
    for (i, c) in line.chars().enumerate() {
        if c == ';' {
            // comment
            break;
        } else {
            if c == ' ' || c == '\t' {
                collecting_chars = false;
            } else if c.is_ascii_alphabetic() {
                collecting_chars = true;
                chars_vec.push(c);
            }
        }

        if !collecting_chars && chars_vec.len() > 0 {
            let s: String = chars_vec.iter().collect();

            opcode = match &*s {
                "syscall" => Some(Opcode::Syscall),
                _ => None,
            };

            //chars_vec.clear();
            break;
        }
    }

    if opcode.is_none() {
        let s: String = chars_vec.iter().collect();
        return Err(ParseError::InvalidInstruction(s));
    }

    return Ok(Instruction {
        opcode,
        operands: Vec::new(),
    });
}
