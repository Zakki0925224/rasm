use crate::parse::Instruction;

#[derive(Debug, Clone)]
pub struct SectionNode {
    pub name: String,
    pub global_labels: Vec<String>,
    pub default_instructions: Vec<Instruction>,
    pub labeled_instructions: Vec<(String, Vec<Instruction>)>,
}

impl SectionNode {
    pub fn new(name: String) -> Self {
        return Self {
            name,
            global_labels: Vec::new(),
            default_instructions: Vec::new(),
            labeled_instructions: Vec::new(),
        };
    }
}
