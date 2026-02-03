use std::fs::File;
use std::io::{self, BufReader, BufRead};
use std::path::Path;


#[derive(Debug, Clone)]
pub enum Command {
    // Address: "@sum" -> "sum"
    A(String),
    // Command: Holds (dest, comp, jump) Strings
    C(Option<String>, String, Option<String>),
    // Label: Marks a specific line of byte code to jump to: "(LOOP)" -> LOOP
    L(String)
}

pub struct Parser {
    lines: Box<dyn Iterator<Item = String>>,
}

impl Parser {
    pub fn new(file_path: &Path) -> io::Result<Self> {
        let file = File::open(file_path)?; // We already confirmed it existed
        let reader = BufReader::new(file);

        // This lets us not *actually* do any of this until it maters while taking this implicit
        // logic back to the main assembler so from the perspective of the assembler it gets a box
        // that spits out enumerations consistently
        let iter = reader.lines()
            .map_while(Result::ok)
            .map(|line| {
                let mut cleaned = line.trim();
                if let Some(idx) = cleaned.find("//") {
                    cleaned = &cleaned[..idx];
                }
                cleaned.trim().to_string()
            })
            .filter(|line| !line.is_empty());

        Ok(Parser {lines: Box::new(iter)})
    }
}

impl Iterator for Parser {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        let instruction = self.lines.next()?;

        // Categorize instruction
        // Address
        if instruction.starts_with('@') {
            // TODO:  add a function for converting common reserved words to uppercase
            return Some(Command::A(instruction[1..].to_string()));
        // Label
        } else if instruction.starts_with('(') && instruction.ends_with(')') {
            return Some(Command::L(instruction[1..instruction.len()-1].to_string()));
        }
        // Compute - default
        let mut dest = None;
        let mut jump = None;

        // NOTE: comp instructions are always case insensitive
        let binding = instruction.to_uppercase();
        let parts: Vec<&str> = binding.split('=').collect();

        let comp_jump_part = match parts.len()  {
            1 => parts[0],
            2 => {
                dest = Some(parts[0].to_string());
                parts[1]
            },
            _ => panic!("Syntax Error: Multiple '=' in C-Command")
        };

        let sub_parts: Vec<&str> = comp_jump_part.split(';').collect();
        let comp = sub_parts[0].to_string();

        if sub_parts.len() == 2 {
            jump = Some(sub_parts[1].to_string());
        } else if sub_parts.len() >= 2 {
            panic!("Syntax Error: Multiple ';' in C-Command");
        }

        Some(Command::C(dest, comp, jump))

    }
}
