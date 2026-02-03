use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

//TODO: change u32 to u16 since this is translating down to a specific known 16x CPU
#[derive(Debug, Clone)]
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

// NOTE: enums finish here
pub struct Parser {
    lines: io::Lines<BufReader<File>>,
}

impl Parser {
    pub fn new(file_path: &Path) -> io::Result<Self> {
        let file = File::open(file_path)?;
        Ok(Parser {
            lines: BufReader::new(file).lines(),
        })
    }
}

impl Iterator for Parser  {
    type Item = Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop until a command or EOF
        while let Some(line_result) = self.lines.next() {
            let line = line_result.ok()?;

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

            // Here we define the control flow we will feed to the writer
            match command_word.to_lowercase().to_str() { // no risk of collision for commands
                // # Mathutation
                // ## Negation of what is stored in Memory
                "not" => Some(Cmd::Math(Math::Neg(Neg::Not))), // M=!M
                "neg" => Some(Cmd::Math(Math::Neg(Neg::Neg))), // M=-M
                // ## Binary applies Data to Memory then stores in Memory
                "add" => Some(Cmd::Math(Math::Bin(Bin::Add))), // M=M+D
                "sub" => Some(Cmd::Math(Math::Bin(Bin::Sub))), // M=M-D
                "and" => Some(Cmd::Math(Math::Bin(Bin::And))), // M=M&D
                "or"  => Some(Cmd::Math(Math::Bin(Bin::Or))),  // M=M|D
                // ## Comparions subtacts stored Data from the target in Memory
                //    Always runs D=M-D and prints a label
                "eq" => Some(Cmd::Math(Math::Comp(Comp::Eq))), // D;JEQ
                "gt" => Some(Cmd::Math(Math::Comp(Comp::Gt))), // D;JGT
                "lt" => Some(Cmd::Math(Math::Comp(Comp::Lt))), // D;JLT
                // # Push/Pop
                "push" => {
                    let (segment, offset) = define_segment(
                    parts.next()?,
                    parts.next()?.parse::<u32>().ok());

                    Cmd::Push(segment, offset)
                },
                "pop" => {
                    let (segment, offset) = define_segment(
                    parts.next()?,
                    parts.next()?.parse::<u32>().ok());

                    Cmd::Pop(segment, offset)
                },
                "label" => Some(Cmd::Label(parts.next()?.to_string())),
                "goto"  => Some(Cmd::Goto(parts.next()?.to_string())),
                "if-goto"  => Some(Cmd::If(parts.next()?.to_string())),
                "function" =>  Some(Cmd::Function(
                        parts.next()?.to_string(),
                        parts.next()?.parse::<u32>().ok()?)),

                "call" =>  Some(Cmd::Call(
                        parts.next()?.to_string(),
                        parts.next()?.parse::<u32>().ok()?)),
                "return" => return Some(Cmd::Return),

                // Unknown command?
                _ => panic!("Unknown command: {}", command_word),
            }
        }
        None
    }
}

fn define_segment(segment: &str, offset: u32) -> (Segment, u32) {
    match segment.to_string().to_lowercase(){
        "argument" => (Segment::Argument, offset),
        "local" => (Segment::Local, offset),
        "static" => {
            if offset > 239 {
                panic!("Static offset out of bounds: {}", offset)
            }
            (Segment::Static, offset)
        },
        "constant" => {
            if offset > 32768 {
                panic!("Overflow exception")
            }

            (Segment::Constant, offset)
        },
        "this" => {
            if offset > 1 {
                panic!("Out of bounds this offset: {}", offset)
            }

            (Segment::This, offset)
        },
        "that" => {
            if offset != 0 {
                panic!("Overflow exception: That can only hold 1 pointer")
            }

            (Segment::That, offset)
        },
        "pointer" => {
            if offset > 1 {
                panic!("Pointer cache overflow")
            }

            (Segment::Pointer, offset)
        },
        "temp" => {
            if offset > 8 {
                panic!("Overflow exception")
            }

            (Segment::Temp, offset)
        },
        _ => panic!("Invalid segment name: '{}'", segment.to_string())
    }
}

