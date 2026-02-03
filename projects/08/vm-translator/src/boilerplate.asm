@256
D=A
@SP
M=D // Init stack
@133
0;JMP // Jump to sys.init
@R15 // ENTRY POINT -- LINE 7 --EQ
M=D // saves entry address
@SP // Line 9 is stack pointer decrement
AM=M-1 // Decrement and moves
D=M // reads data
A=A-1 // moves back again
D=M-D // preform comp
M=0 // save zero(FALSE) to memory
@END_EQ // checks if M-D == 0
D;JNE // Jumps to end of comp if not eq
@SP
A=M-1 // Looking at top of stack
M=-1 // Set to TRUE
(END_EQ) // Label count: 1
@R15
A=M // address from saved pointer
0;JMP // unconditional jump
@R15 // ENTRY POINT -- LINE (24-1) 23 -- GT
M=D
@SP
AM=M-1
D=M
A=A-1
D=M-D
M=0 // this code is identical we can make it a function
@END_GT //
D;JLE
@SP
A=M-1
M=-1
(END_GT) // Label count: 2
@R15
A=M
0;JMP
@R15 // ENTRY POINT -- LINE (41-2) 39 -- LT
M=D
@SP
AM=M-1
D=M
A=A-1
D=M-D
M=0
@END_LT
D;JGE
@SP
A=M-1
M=-1
(END_LT) // Label count: 3
@R15
A=M
0;JMP
@5 // ENTRY POINT -- LINE (58-3) 55 -- RETURN
D=A // we use the code for const 5 here. Might be worth to do by hand
@LCL
A=M-D
D=M
@R13
M=D
@SP
AM=M-1
D=M
@ARG
A=M
M=D
D=A
@SP
M=D+1
@LCL
D=M
@R14 // WARN: HERE!!!
AM=D-1
D=M
@THAT
M=D
@R14
AM=M-1
D=M
@THIS
M=D
@R14
AM=M-1
D=M
@ARG
M=D
@R14
AM=M-1
D=M
@LCL
M=D
@R13
A=M
0;JMP
@SP
A=M
M=D
@LCL
D=M
@SP
AM=M+1
M=D
@ARG
D=M
@SP
AM=M+1
M=D
@THIS
D=M
@SP
AM=M+1
M=D
@THAT
D=M
@SP
AM=M+1
M=D
@4
D=A
@R13
D=D+M
@SP
D=M-D
@ARG
M=D
@SP
MD=M+1
@LCL
M=D
@R14
A=M
0;JMP
@0
D=A
@R13
M=D
@sys.init
D=A
@R14
M=D
@RET_ADDRESS_CALL0
D=A
@95
0;JMP
(RET_ADDRESS_CALL0)



// BUG: Don't go past here

    // --- Push / Pop ---
    fn write_push(&mut self, segment: Segment, index: u32) -> io::Result<()> {
        match segment {
            Segment::Constant => {
                writeln!(self.writer, "@{}", index)?;
                self.cache_pointer()?;
            },
            Segment::Static => {
                writeln!(self.writer, "@{}.{}", self.current_file, index)?;
                self.read()?;
            },
            Segment::Temp => {
                writeln!(self.writer, "@{}", 5 + index)?;
                self.read()?;
            },
            Segment::Pointer => {
                writeln!(self.writer, "@{}", 3 + index)?;
                self.read()?;
            },
            // Dynamic segments (LCL, ARG, THIS, THAT)
            seg => {
                self.get_segment_base(seg)?;
                self.read()?;
                writeln!(self.writer, "@{}", index)?;
                writeln!(self.writer, "A=D+A")?;
                self.read()?;
            }
        }
        self.push_d_to_stack()
    }

    fn write_pop(&mut self, segment: Segment, index: u32) -> io::Result<()> {
        match segment {
            Segment::Static => {
                writeln!(self.writer, "@{}.{}", self.current_file, index)?;
                self.cache_pointer()?;
            },
            Segment::Temp => {
                writeln!(self.writer, "@{}", 5 + index)?;
                self.cache_pointer()?;
            },
            Segment::Pointer => {
                writeln!(self.writer, "@{}", 3 + index)?;
                self.cache_pointer()?;
            },
            seg => {
                self.get_segment_base(seg)?;
                self.read()?;
                writeln!(self.writer, "@{}", index)?;
                writeln!(self.writer, "D=D+A")?; // D holds address
            }
        }

        writeln!(self.writer, "@R13")?; // Store target address in R13
        self.write()?;

        self.stack_pop_read()?;
        writeln!(self.writer, "@R13")?;
        writeln!(self.writer, "A=M")?;
        self.write()?;
        Ok(())
    }

    // --- Placeholders for Flow Control (You can fill these based on Java logic) ---
    fn write_label(&mut self, label: &str) -> io::Result<()> {
        writeln!(self.writer, "({}${})", self.current_function, label)
    }

    fn write_goto(&mut self, label: &str) -> io::Result<()> {
        writeln!(self.writer, "@{}${}", self.current_function, label)?;
        writeln!(self.writer, "0;JMP")
    }

    fn write_if(&mut self, label: &str) -> io::Result<()> {
        self.stack_pop_read()?;
        writeln!(self.writer, "@{}${}", self.current_function, label)?;
        writeln!(self.writer, "D;JNE") // Jump if True (Not Equal to 0)
    }

    fn write_function(&mut self, name: &str, n_vars: u32) -> io::Result<()> {
        self.current_function = name.to_string();
        writeln!(self.writer, "({})", name)?;
        // Initialize local vars to 0
        for _ in 0..n_vars {
            writeln!(self.writer, "@0")?;
            self.cache_pointer()?;
            self.push_d_to_stack()?;
        }
        Ok(())
    }

    fn write_call(&mut self, name: &str, n_args: u32) -> io::Result<()> {
        // (Implementation left as exercise or I can provide if requested)
        // Similar to Java: Push returnAddr, LCL, ARG, THIS, THAT...
        Ok(())
    }

    fn write_return(&mut self) -> io::Result<()> {
        // (Implementation similar to Java writeReturn)
        Ok(())
    }




}
