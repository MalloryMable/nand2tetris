use std::fs::File;
use std::io::{self, BufRead, Buf, Reader};
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
                    cleaned = &claned[..idx];
                }
                cleaned.trim().to_string().to_uppercase()
            })
            .filter(|line| !line.is_empty());

        Ok(Parser {lines: Box::new(iter)})
    }
}

impl Iterator for Parser {
    type Item = Command;

    fn next(&mut self) -> Result<Self::Item> {
        // Categorize instruction
        // Address
        if instruction.starts_with('@') {
            Ok(Command::A(instruction[1..].to_string()))
        // Label
        } else if instruction.starts_with('(') && instruction.ends_with(')') {
            Ok(Command::L(instruction[1..instruction.len()-1].to_string()))
        }
        // Compute - default
        let mut dest = None;
        let mut jump = None;

        let parts: Vec<&str> = instruction.split('=').collect();

        let comp_jump_part = match parts.len()  {
            1 => parts[0],
            2 => {
                dest = Some(parts[0].to_string());
                parts[1]
            },
            _ =>  {}// TODO: throw Error
        };

        let sub_parts: Vec<&str> = comp_jump_part.split(';').collect();
        let comp = sub_parts[0].to_string();

        if sub_parts.len() == 2 {
            jump = Some(sub_parts[1].to_string());
        } else {
            // TODO: throw Error
        }

        Some(Command::C(dest, comp, jump));

    }
}
