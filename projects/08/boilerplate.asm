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

@SP // comp_true
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

@SP // AT CALL
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

@0 // self.call("sys.init");
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



