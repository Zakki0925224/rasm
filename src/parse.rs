// opcodes
const OP_SYSCALL: [u8; 2] = [0x0f, 0x05];
//const OP_MOV_RM32_IMM32: [u8; 2] = [0x48, 0xc7];
const OP_NOP: [u8; 1] = [0x90];

#[derive(Debug, Clone)]
pub enum Opcode {
    Syscall,
    //MovRm32Imm32, // copy imm32 to rm32
    Nop,
}

impl Opcode {
    pub fn get_opcode(&self) -> Vec<u8> {
        return match self {
            Opcode::Syscall => OP_SYSCALL.to_vec(),
            Opcode::Nop => OP_NOP.to_vec(),
            _ => unreachable!(),
        };
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Directive {
    Global(Vec<String>),
    Section(String),
}

#[derive(Debug, Clone)]
pub enum LineToken {
    Invalid,
    Empty,
    Comment,
    Instruction(Instruction),
    Directive(Directive),
    Label(String),
}

pub fn parse(line: &str) -> LineToken {
    let line = line.trim();

    if line.len() == 0 {
        return LineToken::Empty;
    }

    if line.chars().nth(0).unwrap() == ';' {
        return LineToken::Comment;
    }

    // word splitted by space
    let words: Vec<&str> = line.split(" ").collect();
    match words[0] {
        "global" | "section" => {
            if words.len() == 1 {
                return LineToken::Invalid;
            }

            let symbols = (&words[1..])
                .to_vec()
                .iter()
                .map(|w| w.to_string())
                .collect();
            let directive = match words[0] {
                "global" => Directive::Global(symbols),
                "section" => Directive::Section(symbols[0].clone()),
                _ => unreachable!(),
            };

            return LineToken::Directive(directive);
        }
        w => {
            if words.len() == 1 && w.ends_with(':') {
                return LineToken::Label(w.replace(":", ""));
            }

            // parse instructions
            let (opcode, operands) = match w {
                "nop" => (Opcode::Nop, vec![]),
                "syscall" => (Opcode::Syscall, vec![]),
                _ => return LineToken::Invalid,
            };

            return LineToken::Instruction(Instruction { opcode, operands });
        }
    }
}

#[derive(Debug)]
pub enum CheckErrorType {
    InvalidInstruction,
    InvalidSectionName,
}

#[derive(Debug)]
pub enum CheckResult {
    Ok,
    Error {
        at: usize,
        error_type: CheckErrorType,
    },
}

pub fn check_tokens(tokens: &Vec<LineToken>) -> CheckResult {
    for (i, token) in tokens.iter().enumerate() {
        match token {
            LineToken::Empty | LineToken::Comment => continue,
            LineToken::Invalid => {
                return CheckResult::Error {
                    at: i,
                    error_type: CheckErrorType::InvalidInstruction,
                };
            }
            LineToken::Directive(directive) => match directive {
                Directive::Section(section_name) => {
                    if !section_name.starts_with('.')
                        || (section_name.starts_with('.') && section_name.len() == 1)
                    {
                        return CheckResult::Error {
                            at: i,
                            error_type: CheckErrorType::InvalidSectionName,
                        };
                    }
                }
                _ => (),
            },
            // LineToken::Instruction { opcode, operands } => todo!(),
            // LineToken::Label(_) => todo!(),
            _ => (),
        }
    }

    return CheckResult::Ok;
}
