use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub enum Cmd {
    // Actual computation/math opperations
    Math(Math),

    // Pop/Pop to/from the register at the offset from a given segment
    Push(Segment, u32),
    Pop(Segment, u32), // NOTE: not sure I like wrapping this

    // Program flow
    Label(String),
    Goto(String),
    If(String), // if-goto

    // Function calling
    Function(String, u32), // function name nVars
    Return,
    Call(String, u32), // function name nArgs
}

pub enum Segment {
    Argument, // Dynamically alocated
    Local, // Dynamically alocated
    Static, // Allocated by the VM hard limits of ..=(255 - 16 = 239)
    Constant, // ..=32767
    This, //  ..2
    That, // ..1
    Pointer, // < 2(aligns with This and That)
    Temp, // ..8
}

pub enum Math {
    Neg(Neg), // Negating commands
    Bin(Bin), // Binary opperations
    Comp(Comp), // Comparions opperations
}

pub enum Neg { // Recall from software defintion that -M and !M are not equal
    Not,
    Neg,
}

pub enum Bin {
    Add,
    Sub,
    And,
    Or,
}

pub enum Comp {
    Eq,
    Gt,
    Lt,
}

pub struct Parser {
    lines: io::Lines<BufReader<File>>,
    line_number: usize,
    error: Option<String>,
}

impl Parser {
    pub fn new(file_path: &Path) -> io::Result<Self> {
        let file = File::open(file_path)?;
        Ok(Parser {
            lines: BufReader::new(file).lines(),
            line_number: 0,
            error: None,
        })
    }

    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    fn parse_line(&self, command_word: &str, mut parts: std::str::SplitWhitespace) -> Result<Cmd, String> {
        match command_word.to_lowercase().as_str() {
            // # Mathutation
            // ## Negation of what is stored in Memory
            "not" => Ok(Cmd::Math(Math::Neg(Neg::Not))), // M=!M
            "neg" => Ok(Cmd::Math(Math::Neg(Neg::Neg))), // M=-M
            // ## Binary applies Data to Memory then stores in Memory
            "add" => Ok(Cmd::Math(Math::Bin(Bin::Add))), // M=M+D
            "sub" => Ok(Cmd::Math(Math::Bin(Bin::Sub))), // M=M-D
            "and" => Ok(Cmd::Math(Math::Bin(Bin::And))), // M=M&D
            "or"  => Ok(Cmd::Math(Math::Bin(Bin::Or))),  // M=M|D
            // ## Comparions subtacts stored Data from the target in Memory
            //    Always runs D=M-D and prints a label
            "eq" => Ok(Cmd::Math(Math::Comp(Comp::Eq))), // D;JEQ
            "gt" => Ok(Cmd::Math(Math::Comp(Comp::Gt))), // D;JGT
            "lt" => Ok(Cmd::Math(Math::Comp(Comp::Lt))), // D;JLT

            // # Push/Pop
            "push" => {
                let seg = parts.next().ok_or("Missing segment argument")?;
                let idx_str = parts.next().ok_or("Missing index argument")?;
                let idx = idx_str.parse::<u32>().map_err(|_| "Invalid index number")?;

                let (segment, offset) = define_segment(seg, idx)?;
                Ok(Cmd::Push(segment, offset))
            },
            "pop" => {
                let seg = parts.next().ok_or("Missing segment argument")?;
                let idx_str = parts.next().ok_or("Missing index argument")?;
                let idx = idx_str.parse::<u32>().map_err(|_| "Invalid index number")?;

                let (segment, offset) = define_segment(seg, idx)?;
                Ok(Cmd::Pop(segment, offset))
            },

            "label" => Ok(Cmd::Label(parts.next().ok_or("Missing label name")?.to_string())),
            "goto"  => Ok(Cmd::Goto(parts.next().ok_or("Missing goto label")?.to_string())),
            "if-goto"  => Ok(Cmd::If(parts.next().ok_or("Missing if-goto label")?.to_string())),

            "function" => {
                let name = parts.next().ok_or("Missing function name")?.to_string();
                let vars = parts.next().ok_or("Missing var count")?
                                .parse::<u32>().map_err(|_| "Invalid var count")?;
                Ok(Cmd::Function(name, vars))
            },
            "call" => {
                let name = parts.next().ok_or("Missing function name")?.to_string();
                let args = parts.next().ok_or("Missing arg count")?
                                .parse::<u32>().map_err(|_| "Invalid arg count")?;
                Ok(Cmd::Call(name, args))
            },
            "return" => Ok(Cmd::Return),

            // Unknown command?
            _ => Err(format!("Unknown command: {}", command_word)),
        }
    }

}

impl Iterator for Parser  {
   type Item = Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        // If we previously hit an error we stop immediately
        if self.error.is_some() {
            return None;
        }

        // Loop until a command or EOF
        while let Some(line_result) = self.lines.next() {
            self.line_number += 1;

            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    self.error = Some(format!("IO Error: {}", e));
                    return None;
                }
            };

            // Strip comments
            let content = match line.find("//") {
                Some(idx) => &line[..idx],
                None => &line,
            };

            // Tokenize
            let mut parts = content.split_whitespace();

            // Get the first word(defines the structure of the command)
            let command_word = match parts.next() {
                Some(word) => word,
                None => continue, // empty line, skip
            };

            // Try to parse using the helper
            match self.parse_line(command_word, parts) {
                Ok(cmd) => return Some(cmd),
                Err(msg) => {
                    self.error = Some(format!("Line {}: {}", self.line_number, msg));
                    return None;
                }
            }
        }
        None
    }
}

fn define_segment(segment: &str, offset: u32) -> Result<(Segment, u32), String> {
    match segment.to_lowercase().as_str() {
        "argument" => Ok((Segment::Argument, offset)),
        "local" => Ok((Segment::Local, offset)),
        "static" => {
            if offset > 239 {
                return Err(format!("Static offset out of bounds: {}", offset));
            }
            Ok((Segment::Static, offset))
        },
        "constant" => {
            if offset > 32768 {
                return Err("Overflow exception".to_string());
            }
            Ok((Segment::Constant, offset))
        },
        "this" =>  Ok((Segment::This, offset)),
        "that" =>  Ok((Segment::That, offset)),
        "pointer" => {
            if offset > 1 {
                return Err("Pointer cache overflow".to_string());
            }
            Ok((Segment::Pointer, offset))
        },
        "temp" => {
            if offset > 7 {
                return Err("Overflow exception".to_string());
            }
            Ok((Segment::Temp, offset))
        },
        _ => Err(format!("Invalid segment name: '{}'", segment))
    }
}
