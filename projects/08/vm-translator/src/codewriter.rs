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
            current_file: "Bootstrap".to_string(), // Default until set_file_name is called
            current_function: "Sys.init".to_string(),
            label_map: HashMap::new(),
        })
    }

    pub fn set_file_name(&mut self, name: &str) {
        self.current_file = name.to_string();
    }

    // Main entry point to write a command
    pub fn write_command(&mut self, cmd: Cmd) -> io::Result<()> {
        match cmd {
            Cmd::Math(math) => self.write_arithmetic(math),
            Cmd::Push(seg, idx) => self.write_push(seg, idx),
            Cmd::Pop(seg, idx) => self.write_pop(seg, idx),
            Cmd::Label(label) => self.write_label(&label),
            Cmd::Goto(label) => self.write_goto(&label),
            Cmd::If(label) => self.write_if(&label),
            Cmd::Function(name, n_vars) => self.write_function(&name, n_vars),
            Cmd::Call(name, n_args) => self.write_call(&name, n_args),
            Cmd::Return => self.write_return(),
        }
    }

    fn write_arithmetic(&mut self, math: Math) -> io::Result<()> {
        self.stack_peak()?; // [@SP, A=M-1] (Moves to filled slot)
        match math {
            Math::Neg(op) => {
                match op {
                    Neg::Not => writeln!(self.writer, "M=!M")?,
                    Neg::Neg => writeln!(self.writer, "M=-M")?,
                }
            },
            Math::Bin(op) => {
                self.read()?; // D=M (Pop top value) (Save 'y' ->D)
                self.stack_pop(); // [@SP, AM=M-1] (pop 'y')
                self.peak()?; // A=M-1 ("peak" at address of second value)

                match op { // Notice we preform this in place
                    Bin::Add => writeln!(self.writer, "M=M+D")?,
                    Bin::Sub => writeln!(self.writer, "M=M-D")?,
                    Bin::And => writeln!(self.writer, "M=M&D")?,
                    Bin::Or  => writeln!(self.writer, "M=M|D")?,
                }
            },
            Math::Comp(op) => {
                // Jump to bootstrap line then back to the line after jumping
                let label = self.label("COMP_RTRN");

                writeln!(self.writer, "@{}", label)?;
                self.cache_pointer()?;
                match op {
                    Comp::Eq => writeln!(self.writer, "@ENTER_EQ")?,
                    Comp::Gt => writeln!(self.writer, "@ENTER_GT")?,
                    Comp::Lt => writeln!(self.writer, "@ENTER_LT")?,
                }
                self.goto()?;
                writeln!(self.writer, "({})", label)?;
            }
        }
        Ok(())
    }

    fn write_push(&mut self, seg: Segment, index: u16) -> io::Result<()> {
        match seg {
            Segment::Constant => self.cache_const(index),
            Segment::Local | Segment::Argument | Segment::This | Segment::That => {
                self.get_segment_base(seg)?;
                self.read()?;
                writeln!(self.writer, "@{}", index)?;
                writeln!(self.writer, "A=D+A")?; // Address = Base + Index
                self.read();   // D = *Address
            },
            Segment::Static => {
                writeln!(self.writer, "@{}.{}", self.current_file, index)?;
                self.read()?;
            },
            Segment::Temp => {
                writeln!(self.writer, "@{}", 5 + index)?;
                self.read();
            },
            Segment::Pointer => {
                let label = if index == 0 { "THIS" } else { "THAT" };
                writeln!(self.writer, "@{}", label)?;
                self.read();
            },
        }
        self.stack_push()
    }

    fn write_pop(&mut self, seg: Segment, index: u16) -> io::Result<()> {
        match seg {
            Segment::Constant => panic!("Cannot pop constant"),
            Segment::Local | Segment::Argument | Segment::This | Segment::That => {
                // Address = Base + Index
                self.get_segment_base(seg)?;
                self.read()?;
                writeln!(self.writer, "@{}", index)?;
                writeln!(self.writer, "D=D+A")?;

                // Store address in R13
                writeln!(self.writer, "@R13")?;
                self.write();

                self.stack_pop_read()?;

                // Write D to *R13
                writeln!(self.writer, "@R13")?;
                self.deref()?;
                self.write()?;
            },

            Segment::Static => {
                self.stack_pop_read()?;
                writeln!(self.writer, "@{}.{}", self.current_file, index)?;
                self.write()?;
            },
            Segment::Temp => {
                self.stack_pop_read()?;
                writeln!(self.writer, "@{}", 5 + index)?;
                writeln!(self.writer, "M=D")?;
            },
            Segment::Pointer => {
                self.stack_pop_read()?;
                let label = if index == 0 { "THIS" } else { "THAT" };
                writeln!(self.writer, "@{}", label)?;
                writeln!(self.writer, "M=D")?;
            },
        }
        Ok(())
    }
    // TODO: write_label(&label)
    // TODO: write_goto(&label)
    // TODO: write_if(&label)
    fn write_function(&mut self, name: &str,n_vars: u32) -> io::Result<()> {
        Ok(())

    }

    fn write_call(&mut self, name: &str, n_args: u32) -> io::Result<()> {
        let label = self.label("RTRN_CALL");

        self.cache_const(n_args)?; //[@{n_args}, D=A]
        writeln!(self.writer, "@R13")?;
        self.write()?; // M=D
        writeln!(self.writer, "@{}", name)?;
        self.cache_pointer()?; // A=D
        writeln!(self.writer, "@R14")?;
        self.write(); // M=D

        writeln!(self.writer, "@{}", label)?;
        self.cache_pointer()?;

        writeln!(self.writer, "@ENTER_CALL")?;
        self.goto();

        writeln!(self.writer, "({})", label)
    }

    fn write_return(&mut self) -> io::Result<()> {
        writeln!(self.writer, "@ENTER_RTRN")?;
        self.goto()
    }

    fn label(&mut self, prefix: &str) -> String {
        let count = self.label_map.entry(prefix.to_string()).or_insert(0);
        let label = format!("{}{}", prefix, count);
        *count += 1;

        label
    }

    // ---- Helper Functions ---
    // TODO: @R13 --> at_frame_pointer
    // TODO: @R14 --> at_rtrn_pointer
    fn at_stack_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "@SP") }
    fn peak(&mut self) -> io::Result<()> { writeln!(self.writer, "A=M-1") } // look at top
    fn read(&mut self) -> io::Result<()> { writeln!(self.writer, "D=M") } // Read
    fn write(&mut self) -> io::Result<()> { writeln!(self.writer, "M=D") } // Write
    fn cache_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "D=A") } // pointer
    fn deref(&mut self) -> io::Result<()> { writeln!(self.writer, "A=M") } // Follow pointer
    fn goto(&mut self) -> io::Result<()> { writeln!(self.writer, "0;JMP") } // Unconditional jump


    fn stack_peak(&mut self) -> io::Result<()> {
        self.at_stack_pointer()?; // @SP
        self.peak() // A=M-1
    }

    fn stack_pop(&mut self) -> io::Result<()> {
        self.at_stack_pointer()?; // [@SP]
        writeln!(self.writer, "AM=M-1")
    }

    fn stack_pop_read(&mut self) -> io::Result<()> {
        self.stack_pop(); // [@SP, AM=M-1]
        self.read(); // D=M
    }

    fn stack_push(&mut self) -> io::Result<()> {
        self.at_stack_pointer()?; // @SP
        self.deref()?; // A=M (This is the only time we go straight to the empty register)
        self.write()?; // M=D (Saves data to the top of the stack)
        self.at_stack_pointer()?; // Back to SP
        writeln!(self.writer, "M=M+1") // Pointed to the new empty position
    }

    fn get_segment_base(&mut self, seg: Segment) -> io::Result<()> {
        match seg {
            Segment::Local => writeln!(self.writer, "@LCL"),
            Segment::Argument => writeln!(self.writer, "@ARG"),
            Segment::This => writeln!(self.writer, "@THIS"),
            Segment::That => writeln!(self.writer, "@THAT"),
            _ => panic!("Not a dynamic segment"),
        }
    }

    fn boostrap(&mut self) -> io::Result<()> {

        // STACK INIT
        self.cache_const(256)?;
        self.at_stack_pointer()?;
        self.write()?;
        writeln!(self.writer, "@BOOTED")?;
        self.goto()?;

        // COMPARISON
        // EQ
        writeln!(self.writer, "(ENTER_EQ)")?;
        self.enter_macro()?;
        writeln!(self.writer, "@END_EQ")?;
        writeln!(self.writer, "D;JNE")?; // Jumps if a-b != 0 (we assumed false and were right)
        self.comp_true()?;
        writeln!(self.writer, "(END_EQ)")?;
        self.exit_macro()?;
        // GT
        writeln!(self.writer, "(ENTER_GT)")?;
        self.enter_macro()?;
        writeln!(self.writer, "@END_GT")?;
        writeln!(self.writer, "D;JLE")?; // Jumps if a-b <= 0
        self.comp_true()?;
        writeln!(self.writer, "(END_GT)")?;
        self.exit_macro()?;
        // LT
        writeln!(self.writer, "(ENTER_LT)")?;
        self.enter_macro()?;
        writeln!(self.writer, "@END_LT")?;
        writeln!(self.writer, "D;JGE")?; // Jumps if a-b >= 0
        self.comp_true()?;
        writeln!(self.writer, "(END_LT)")?;
        self.exit_macro()?;

        // RETURN STATEMENT
        writeln!(self.writer, "(ENTER_RTRN)")?;
        // Saves *(*Local-5) to the return register
        self.cache_const(5);
        self.get_segment_base(Segment::Local)?;
        writeln!(self.writer, "A=M-D")?; // $LCL-5 is our new address
        self.read()?;
        writeln!(self.writer, "@R13")?; // Return address chache
        self.write()?;

        self.stack_pop_read()?;
        self.get_segment_base(Segment::Argument)?;
        self.deref()?; // A=M
        self.write()?; // Saves top of stack to arg
        self.cache_pointer()?; // Saves a pointer to Arg

        self.at_stack_pointer()?;
        writeln!(self.writer, "M=D+1")?; // Put the next viable arg register on the stack

        // Back to the top of stack
        self.get_segment_base(Segment::Local)?;
        self.read()?;

        // Restores each segment from call pointer(R14)
        self.frame_pop(Segment::That)?;
        self.frame_pop(Segment::This)?;
        self.frame_pop(Segment::Argument)?;
        self.frame_pop(Segment::Local)?;

        // Jump to return address
        writeln!(self.writer, "@R13")?;
        self.deref()?;
        self.goto()?;

        // CALL
        writeln!(self.writer, "(ENTER_CALL)")?;
        // Pushes return address to top of stack
        self.stack_push();

        self.frame_stack_push(Segment::Local)?;
        self.frame_stack_push(Segment::Argument)?;
        self.frame_stack_push(Segment::This)?;
        self.frame_stack_push(Segment::That)?;

        self.cache_const(5)?;
        writeln!(self.writer, "@R13")?;
        writeln!(self.writer, "D=D+M")?;
        self.at_stack_pointer()?;
        writeln!(self.writer, "D=D-M")?;
        self.get_segment_base(Segment::Argument)?;
        self.write()?;

        self.at_stack_pointer()?;
        writeln!(self.writer, "MD=M+1")?;
        self.get_segment_base(Segment::Local)?;
        self.write()?;

        // Jump to stored return address
        writeln!(self.writer, "@R14")?;
        self.deref()?;
        self.goto();
        writeln!(self.writer, "(BOOTED)")
    }

    fn enter_macro(&mut self) -> io::Result<()> {
        writeln!(self.writer, "@R15")?;
        self.save()?; // M=D
        self.stack_pop_read()?; // [@SP, AM=M-1, D=M] //NOTE: Can we make this DAM=M-1
        self.peak()?; // A=M-1
        writeln!(self.writer, "D=M-D")?; // Saves the comparison
        writeln!(self.writer, "M=0") // Assume false
    }

    fn comp_true(&mut self) -> io::Result<()> {
        self.stack_peak()?; // [@SP, A=M-1] (change in place)
        writeln!(self.writer, "M=-1") // Set true
    }

    fn exit_macro(&mut self) -> io::Result<()> {
        writeln!(self.writer, "@R15")?;
        self.deref()?; // A=M
        self.goto() // 0;JMP
    }

    fn frame_pop(&mut self, segment: Segment) -> io::Result<()> {
        writeln!(self.writer, "@R14")?; // Call target address
        writeln!(self.writer, "AM=M-1")?; // Pop last segment
        self.read()?;
        self.get_segment_base(segment)?;
        self.write()
    }

    fn frame_stack_push(&mut self, segment: Segment) -> io::Result<()> {
        self.get_segment_base(segment)?;
        self.read()?;

        self.at_stack_pointer()?;
        writeln!(self.writer, "AM=M+1")?;
        self.write() // Push segment to address stored in stack
    }

    fn cache_const(&mut self, val: u32) -> io::Result<()> {
        writeln!(self.writer, "@{}", val)?;
        self.cache_pointer()
    }
}
