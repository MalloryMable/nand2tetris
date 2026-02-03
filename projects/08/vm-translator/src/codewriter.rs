// codewriter.rs
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::collections::HashMap;

// Import types from parser
use crate::parser::{Cmd, Segment, Math, Neg, Bin, Comp};

pub struct CodeWriter {
    writer: BufWriter<File>,
    current_file: String,
    current_function: String,
    label_counter: u32,
    call_counter: HashMap<String, u32>,
}

impl CodeWriter {
    pub fn new(output_path: &Path) -> io::Result<Self> {
        let file = File::create(output_path)?;
        Ok(CodeWriter {
            writer: BufWriter::new(file),
            current_file: "Bootstrap".to_string(), // Default until set_file_name is called
            current_function: "Sys.init".to_string(),
            label_counter: 0,
            call_counter: HashMap::new(),
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

    // --- Arithmetic ---
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
                // TODO: Jump to bootstrap line
                // cache return address(get current line(?) add a static offset to the line right
                // after jump invocation)
                // WARN: Critical
                match op { // Each of these will set @ to magic numbers
                    Comp::Eq => writeln!(self.writer, "@{}\nD;JEQ", true_label)?,
                    Comp::Gt => writeln!(self.writer, "@{}\nD;JGT", true_label)?,
                    Comp::Lt => writeln!(self.writer, "@{}\nD;JLT", true_label)?,
                }
                self.goto()?;
            }
        }
        Ok(())
    }

    // --- Helper Methods (Translating your private helpers) ---

    fn at_stack_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "@SP") }
    fn peak(&mut self) -> io::Result<()> { writeln!(self.writer, "A=M-1") } // look at top
    fn read(&mut self) -> io::Result<()> { writeln!(self.writer, "D=M") } // Read
    fn write(&mut self) -> io::Result<()> { writeln!(self.writer, "M=D") } // Write
    fn cache_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "D=A") } // pointer
    fn to_pointer(&mut self) -> io::Result<()> { writeln!(self.writer, "A=M") } // Follow pointer
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
        self.to_pointer()?; // A=M (This is the only time we go straight to the empty register)
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

    // TODO: Make a tool for saving/editing magic numbers for bootstrap
    // sys.init call --> 113
    // return_inner --> R15
    // equals --> 7
    // gt --> 23
    // lt --> 39
    // return --> 55
    // call --> 96

    ///// TODO: Better structure for this
    fn boostrap(&mut self) -> io::Result<()> {

        // STACK INIT
        writeln!(self.writer, "@256")?;
        self.cache_pointer()?;
        self.at_stack_pointer()?;
        self.write()?;
        writeln!(self.writer, "@133")?;
        self.goto()?;

        // COMPARISON
        // EQ
        self.enter_macro()?;
        writeln!(self.writer, "@END_EQ")?;
        writeln!(self.writer, "D;JNE")?; // Jumps if a-b != 0 (we assumed false and were right)
        self.comp_true()?;
        writeln!(self.writer, "(END_EQ)")?;
        self.exit_macro()?;
        // GT
        self.enter_macro()?;
        writeln!(self.writer, "@END_GT")?;
        writeln!(self.writer, "D;JLE")?; // Jumps if a-b <= 0
        self.comp_true()?;
        writeln!(self.writer, "(END_GT)")?;
        self.exit_macro()?;
        // LT
        self.enter_macro()?;
        writeln!(self.writer, "@END_LT")?;
        writeln!(self.writer, "D;JGE")?; // Jumps if a-b >= 0
        self.comp_true()?;
        writeln!(self.writer, "(END_LT)")?;
        self.exit_macro()?;

        // RETURN STATEMENT
        writeln!(self.writer, "@5")?; // NOTE: Magic number
        self.cache_pointer()?;
        self.get_segment_base(Segment::Local)?;
        writeln!(self.writer, "A=M-D")?; // $LCL-5 is our new address
        self.read()?;
        writeln!(self.writer, "@R13")?; // Return address chache
        self.write()?;

        self.stack_pop_read()?;
        self.get_segment_base(Segment::Argument)?;
        self.to_pointer()?; // A=M
        self.write()?; // Saves top of stack to arg
        self.cache_pointer()?;
        self.at_stack_pointer()?;
        writeln!(self.writer, "M=D+1")?; // increment from saved arg pointer
        self.get_segment_base(Segment::Local)?;
        self.read()?;

        self.frame_pop(Segment::That)?;
        self.frame_pop(Segment::This)?;
        self.frame_pop(Segment::Argument)?;
        self.frame_pop(Segment::Local)?;

        writeln!(self.writer, "@R13")?;
        self.to_pointer()?;
        self.goto()?;

        // CALL
        self.frame_stack_push(Segment::Local)?;
        self.frame_stack_push(Segment::Argument)?;
        self.frame_stack_push(Segment::This)?;
        self.frame_stack_push(Segment::That)?;
        writeln!(self.writer, "@4")?; // NOTE: const
        writeln!(self.writer, "D=A")?;
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
        writeln!(self.writer, "@R14")?;
        self.at_pointer()?;
        self.goto()
    }

    fn enter_macro(&mut self) -> io::Result<()> {
        writeln!(self.writer, "@R15")?;
        self.save()?; // M=D
        self.stack_pop_read()?; // [@SP, AM=M-1, D=M] //NOTE: Can we make this DAM=M-1
        self.peak()?; // A=M-1
        writeln!(self.writer, "D=M-D")?; // Saves the comparison
        writeln!(self.writer, "M=0")? // Assume false
    }

        fn comp_true(&mut self) -> io::Result<()> {
        self.stack_peak()?; // [@SP, A=M-1] (change in place)
        writeln!(self.writer, "M=-1")? // Set true
    }

    fn exit_macro(&mut self) -> io::Result<()> {
        writeln!(self.writer, "@R15")?;
        self.to_pointer()?; // A=M
        self.goto() // 0;JMP
    }

    fn frame_pop(&mut self, segment: Segment) -> io::Result<()> {
        writeln!(self.writer, "@R14")?; // Call target address
        writeln!(self.writer, "AM=M-1")?;
        self.read()?;
        self.get_segment_base(segment)?;
        self.write()?;
    }

    fn frame_stack_push(&mut self, segment: Segment) -> io::Result<()> {
        self.at_stack_pointer()?;
        self.to_pointer()?;
        self.write()?; // Notice the first part of a push
        self.get_segment_base(Segment::Local)?;
        self.at_stack_pointer()?;
        writeln!(self.writer, "AM=M+1")?;
        self.read()
    }




//TODO: Get other files
//WARN: Ending missing see other file

}
