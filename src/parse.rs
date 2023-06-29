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
pub enum Directive {
    Global(Vec<String>),
    Section(String),
}

#[derive(Debug, Clone)]
pub enum LineNode {
    Invalid,
    Empty,
    Comment,
    Instruction { opcode: Opcode, operands: Vec<u8> },
    Directive(Directive),
    Label(String),
}

pub fn parse(line: &str) -> LineNode {
    let line = line.trim();

    if line.len() == 0 {
        return LineNode::Empty;
    }

    if line.chars().nth(0).unwrap() == ';' {
        return LineNode::Comment;
    }

    // word splitted by space
    let words: Vec<&str> = line.split(" ").collect();
    match words[0] {
        "global" | "section" => {
            if words.len() == 1 {
                return LineNode::Invalid;
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

            return LineNode::Directive(directive);
        }
        w => {
            if words.len() == 1 && w.ends_with(':') {
                return LineNode::Label(w.replace(":", ""));
            }

            // parse instructions
            let (opcode, operands) = match w {
                "nop" => (Opcode::Nop, vec![]),
                "syscall" => (Opcode::Syscall, vec![]),
                _ => return LineNode::Invalid,
            };

            return LineNode::Instruction { opcode, operands };
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

pub fn check_nodes(nodes: &Vec<LineNode>) -> CheckResult {
    for (i, node) in nodes.iter().enumerate() {
        match node {
            LineNode::Empty | LineNode::Comment => continue,
            LineNode::Invalid => {
                return CheckResult::Error {
                    at: i,
                    error_type: CheckErrorType::InvalidInstruction,
                };
            }
            LineNode::Directive(directive) => match directive {
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
            // LineNode::Instruction { opcode, operands } => todo!(),
            // LineNode::Label(_) => todo!(),
            _ => (),
        }
    }

    return CheckResult::Ok;
}
