use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::collections::HashMap;

use crate::parser::{Cmd, Segment, Math, Neg, Bin, Comp};

pub struct CodeWriter {
    writer: BufWriter<File>,
    current_file: String,
    current_function: String,
    label_map: HashMap<String, u32>,
}

impl CodeWriter {
    pub fn new(output_path: &Path) -> io::Result<Self> {
        let file = File::create(output_path)?;
        Ok(CodeWriter {
            writer: BufWriter::new(file),
            current_file: "Sys".to_string(), // Default until set_file_name is called
            current_function: "init".to_string(),
            label_map: HashMap::new(),
        })
    }

    pub fn set_file_name(&mut self, name: &str) {
        self.current_file = name.to_string();
    }

    // Main entry point to write a command
    pub fn command(&mut self, cmd: Cmd) -> io::Result<()> {
        match cmd {
            Cmd::Math(math) => self.arithmetic(math),
            Cmd::Push(seg, idx) => self.push(seg, idx),
            Cmd::Pop(seg, idx) => self.pop(seg, idx),
            Cmd::Label(label) => self.label(&label),
            Cmd::Goto(label) => self.goto(&label),
            Cmd::If(label) => self.if_fn(&label),
            Cmd::Function(name, n_vars) => self.function(&name, n_vars),
            Cmd::Call(name, n_args) => self.call(&name, n_args),
            Cmd::Return => self.return_fn(),
        }
    }

    fn arithmetic(&mut self, math: Math) -> io::Result<()> {
        match math {
            Math::Neg(op) => {
                self.stack_peak()?;
                match op {
                    Neg::Not => writeln!(self.writer, "M=!M")?,
                    Neg::Neg => writeln!(self.writer, "M=-M")?,
                }
            },
            Math::Bin(op) => {
                self.stack_pop_read()?; // [@SP, AM=M-1] (pop 'y')
                self.peak()?; // A=A-1 ("peak" at address of second value)

                match op { // Notice we preform this in place
                    Bin::Add => writeln!(self.writer, "M=D+M")?,
                    Bin::Sub => writeln!(self.writer, "M=M-D")?,
                    Bin::And => writeln!(self.writer, "M=M&D")?,
                    Bin::Or  => writeln!(self.writer, "M=M|D")?,
                }
            },
            Math::Comp(op) => {
                // Jump to bootstrap line then back to the line after jumping
                let label = self.internal_label("COMP_RTRN");
                self.at(&label)?;

                self.cache_pointer()?;
                match op {
                    Comp::Eq => writeln!(self.writer, "@ENTER_EQ")?,
                    Comp::Gt => writeln!(self.writer, "@ENTER_GT")?,
                    Comp::Lt => writeln!(self.writer, "@ENTER_LT")?,
                }
                self.jump()?;
                writeln!(self.writer, "({})", label)?;
            }
        }
        Ok(())
    }

    fn push(&mut self, seg: Segment, index: u32) -> io::Result<()> {

        match seg {
            Segment::Constant => {self.cache_const(index)?;},
            Segment::Local | Segment::Argument | Segment::This | Segment::That => {
                self.at_segment(seg)?;
                self.read()?;
                self.at(format!("{}", index).as_str())?;
                writeln!(self.writer, "A=D+A")?; // Address = Base + Index
                 self.read()?;   // D = *Address
            },
            Segment::Static => {
                self.at(format!( "{}.{}", self.current_file, index).as_str())?;
                 self.read()?;
            },
            Segment::Temp => {
                self.at(format!( "{}", 5 + index).as_str())?;
                self.read()?;
            },
            Segment::Pointer => {
                let label = if index == 0 { "THIS" } else { "THAT" };
                self.at(label)?;
                self.read()?;
            },
        };
        self.stack_push()
    }

    fn pop(&mut self, seg: Segment, index: u32) -> io::Result<()> {
        match seg {
            Segment::Constant => Err(io::Error::other("Cannot pop constant")),
            Segment::Local | Segment::Argument | Segment::This | Segment::That => {
                // Address = Base + Index
                self.at_segment(seg)?;
                self.read()?;
                self.at(format!("{}", index).as_str())?;
                writeln!(self.writer, "D=D+A")?;

                // Store a target pointer in R13
                self.at_rtrn_pointer()?;
                self.write()?;

                self.stack_pop_read()?;

                // Write D to *R13
                self.at_rtrn_pointer()?;
                self.deref()?;
                return self.write();
            },

            Segment::Static => {
                self.stack_pop_read()?;
                self.at(&format!("{}.{}", self.current_file, index))?;
                return self.write();
            },
            Segment::Temp => {
                self.stack_pop_read()?;
                self.at(&format!("{}", 5 + index))?;
                return self.write();
            },
            Segment::Pointer => {
                self.stack_pop_read()?;
                let register = if index == 0 { "THIS" } else { "THAT" };
                self.at(register)?;
                return self.write();
            },
        }
    }

    fn label (&mut self, label: &str) -> io::Result<()> {
        writeln!(self.writer, "({})", self.external_label(label))
    }
    fn goto(&mut self, label: &str) -> io::Result<()> {
        self.at(&self.external_label(label))?;
        self.jump()
    }

    fn if_fn(&mut self, label: &str) -> io::Result<()> {
        self.stack_pop_read()?;
        self.at(&self.external_label(label))?;
        writeln!(self.writer, "D;JNE" ) // Makes sure D != False

    }

    fn function(&mut self, function: &str,n_vars: u32) -> io::Result<()> {
        self.current_function = function.to_string();
        writeln!(self.writer, "({})", function)?;

        for _ in 0..n_vars {
            self.cache_const(0)?;
            self.stack_push()?;
        }
        Ok(())

    }

    fn call(&mut self, function: &str, n_args: u32) -> io::Result<()> {

        let label = self.internal_label("RTRN_CALL");

        self.cache_const(n_args)?; //[@{n_args}, D=A]
        self.at_frame_pointer()?;
        self.write()?; // M=D
        self.at(function)?;
        self.cache_pointer()?; // A=D
        self.at_rtrn_pointer()?;
        self.write()?; // M=D

        self.at(&label)?;
        self.cache_pointer()?;

        self.at("ENTER_CALL")?;
        self.jump()?;

        writeln!(self.writer, "({})", label)
    }

    fn return_fn(&mut self) -> io::Result<()> {
        self.at("ENTER_RTRN")?;
        self.jump()
    }

    // ---- Helper Functions ---
    fn at_stack_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "@SP") }
    fn at_frame_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "@R13") }
    fn at_rtrn_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "@R14") }
    fn at_comp_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "@R15") }
    fn at(&mut self, addr: &str) -> io::Result<()> { writeln!(self.writer, "@{}", addr) }
    fn deref(&mut self) -> io::Result<()> { writeln!(self.writer, "A=M") } // Follow pointer
    fn peak(&mut self) -> io::Result<()> { writeln!(self.writer, "A=A-1") }
    fn jump(&mut self) -> io::Result<()> { writeln!(self.writer, "0;JMP") } // Unconditional jump
    fn cache_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "D=A") }
    fn read(&mut self) -> io::Result<()> { writeln!(self.writer, "D=M") }
    fn write(&mut self) -> io::Result<()> { writeln!(self.writer, "M=D") }

    fn stack_peak(&mut self) -> io::Result<()> {
        self.at_stack_pointer()?; // @SP
        writeln!(self.writer, "A=M-1") // We haven't dereferenced yet
    }

    fn stack_pop(&mut self) -> io::Result<()> {
        self.at_stack_pointer()?; // [@SP]
        writeln!(self.writer, "AM=M-1")
    }

    fn stack_pop_read(&mut self) -> io::Result<()> {
        self.stack_pop()?; // [@SP, AM=M-1]
        self.read() // D=M
    }

    fn stack_push(&mut self) -> io::Result<()> {
        self.at_stack_pointer()?; // @SP
        self.deref()?;
        // writeln!(self.writer, "AM=M+1")?;
        self.write()?; // M=D (Saves data to the top of the stack)
        writeln!(self.writer, "A=M+1")
    }

    fn at_segment(&mut self, seg: Segment) -> io::Result<()> {
        match seg {
            Segment::Local => writeln!(self.writer, "@LCL"),
            Segment::Argument => writeln!(self.writer, "@ARG"),
            Segment::This => writeln!(self.writer, "@THIS"),
            Segment::That => writeln!(self.writer, "@THAT"),
            _ => panic!("Not a dynamic segment"),
        }
    }

    fn external_label(&self, label: &str ) -> String {
         format!("{}${}", self.current_function, label)
    }

    // Internal label writing to avoid name collision
    fn internal_label(&mut self, prefix: &str) -> String {
        let count = self.label_map.entry(prefix.to_string()).or_insert(0);
        let label = format!("{}{}", prefix, count);
        *count += 1;

        label
    }

    pub fn init(&mut self) -> io::Result<()> {

        // STACK INIT
        self.cache_const(256)?;
        self.at_stack_pointer()?;
        self.write()?;
        self.at("BOOTED")?;
        self.jump()?;

        // COMPARISON
        // EQ
        writeln!(self.writer, "(ENTER_EQ)")?;
        self.enter_macro()?;
        self.at("END_EQ")?;
        writeln!(self.writer, "D;JNE")?; // Jumps if a-b != 0 (we assumed false and were right)
        self.comp_true()?;
        writeln!(self.writer, "(END_EQ)")?;
        self.exit_macro()?;
        // GT
        writeln!(self.writer, "(ENTER_GT)")?;
        self.enter_macro()?;
        self.at("END_GT")?;
        writeln!(self.writer, "D;JLE")?; // Jumps if a-b <= 0
        self.comp_true()?;
        writeln!(self.writer, "(END_GT)")?;
        self.exit_macro()?;
        // LT
        writeln!(self.writer, "(ENTER_LT)")?;
        self.enter_macro()?;
        self.at("END_LT")?;
        writeln!(self.writer, "D;JGE")?; // Jumps if a-b >= 0
        self.comp_true()?;
        writeln!(self.writer, "(END_LT)")?;
        self.exit_macro()?;

        // RETURN STATEMENT
        writeln!(self.writer, "(ENTER_RTRN)")?;
        // Saves *(*Local-5) to the return register
        self.cache_const(5)?;
        self.at_segment(Segment::Local)?;
        writeln!(self.writer, "A=M-D")?; // LCL-5 is our new address
        self.read()?;
        self.at_frame_pointer()?; // Return address is set here
        self.write()?;

        self.stack_pop_read()?;
        self.at_segment(Segment::Argument)?;
        self.deref()?; // A=M
        self.write()?; // Saves top of stack to arg
        self.cache_pointer()?;

        // Passes saved arg
        self.at_stack_pointer()?;
        writeln!(self.writer, "M=D+1")?; // Put the next viable arg register on the stack

        // Updates the frame pointer from local
        self.at_segment(Segment::Local)?;
        self.read()?;

        // Restores each segment from frame
        self.frame_pop(Segment::That)?;
        self.frame_pop(Segment::This)?;
        self.frame_pop(Segment::Argument)?;
        self.frame_pop(Segment::Local)?;

        // Jump out of frame
        self.at_frame_pointer()?; // R13
        self.deref()?;
        self.jump()?;

        // CALL
        writeln!(self.writer, "(ENTER_CALL)")?;
        // Pushes return address to top of stack without incrementing
        self.at_stack_pointer()?;
        self.deref()?;
        self.write()?;

        self.frame_stack_push(Segment::Local)?;
        self.frame_stack_push(Segment::Argument)?;
        self.frame_stack_push(Segment::This)?;
        self.frame_stack_push(Segment::That)?;

        // ARG = SP - (n_args + 4) (SP is already 1 behind from not icrementing)
        self.cache_const(4)?;
        self.at_frame_pointer()?; // R13
        writeln!(self.writer, "D=D+M")?;
        self.at_stack_pointer()?;
        writeln!(self.writer, "D=M-D")?;
        self.at_segment(Segment::Argument)?;
        self.write()?;

        // LCL = SP
        self.at_stack_pointer()?;
        writeln!(self.writer, "MD=M+1")?;
        self.at_segment(Segment::Local)?;
        self.write()?;

        // Jump to return address
        self.at_rtrn_pointer()?;
        self.deref()?;
        self.jump()?;
        writeln!(self.writer, "(BOOTED)")?;
        self.call("Sys.init", 0)
    }

    // -- INIT specific helper functions --

    fn enter_macro(&mut self) -> io::Result<()> {
        self.at_comp_pointer()?; // R15
        self.write()?; // M=D
        self.stack_pop_read()?; // [@SP, AM=M-1, D=M] //NOTE: Can we make this DAM=M-1
        self.peak()?; // A=A-1 // writeln!(self.writer, "A=A-1")?;
        writeln!(self.writer, "D=M-D")?; // Saves the comparison
        writeln!(self.writer, "M=0") // Assume false
    }

    fn comp_true(&mut self) -> io::Result<()> {
        self.stack_peak()?; // [@SP, A=M-1] (change in place)
        writeln!(self.writer, "M=-1") // Set true
    }

    fn exit_macro(&mut self) -> io::Result<()> {
        self.at_comp_pointer()?;
        self.deref()?; // A=M
        self.jump() // 0;JMP
    }

    fn frame_pop(&mut self, segment: Segment) -> io::Result<()> {
        self.at_rtrn_pointer()?; // R14
        writeln!(self.writer, "AM=D-1")?; // Pop last segment
        self.read()?;
        self.at_segment(segment)?;
        self.write()
    }

    fn frame_stack_push(&mut self, segment: Segment) -> io::Result<()> {
        self.at_segment(segment)?;
        self.read()?;
        self.stack_push()
    }

    fn cache_const(&mut self, val: u32) -> io::Result<()> {
        self.at(format!("{}", val).as_str())?;
        self.cache_pointer()
    }
}
