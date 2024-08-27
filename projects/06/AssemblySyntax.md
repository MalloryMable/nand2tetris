## Assembly Syntax

This is a highly simplified assymbly language only capable of addition and subtration and control flow.

### Addresses and Opp Code

**Structure**\
The address a given instruction will be performed at is denoted by an @ and a 15 bit register or variable above the desired opp code.
```
    @2000
    M=M+1
```
The above code adds one to the value stored in register 2000 and then jumps to the value stored in A

**Opp code syntax**\
The opp code is composed of 3 optional sections.
A destination, computation instruction for the ALU, and the jump instruction.
Of these only the ALU instruction is mandatory(to ommit in practice simply pass zero).\
*dest **=** comp **;** jump* 

**Destinations**\
The destinations D, A, and M are used when writing opp code and represent both where to pull information from and where it will be written to.
Data before an equal sign denotes destination while after the equal sign denotes operations.
The D represents data a loose register used for saving a vaule for doing math or moving between registers.
The A represents the address defined above the operation.
The M represents the data stored in Memory[A].

**Variables**\
When a new unique string appears after the address symbol(@) a new RAM address is set aside starting at register 16(0x0010).
```
    @i
    M=1
    @3
    D=A
    @i
    M=M+D
```
After the above code is run the value stored in 0x0010 is now 4.

### Labels

Labels are our first bit of high level abstraction as they allow us to offload recall of the specific line number of a given opp code to the assembler.
To declare a new label simp  ly write a string within a paranthetical to a new line\
```
(LOOP)
    M=1
    @LOOP
    0;JMP
```
Notice that by convention labels are written in all caps, and all registers are denoted by capitalized letters.
Using variable syntax for labels points at the 

### Jumps
Jump conditionals check the flags output by the alu after the computation step and if the condition is satisfied move to the location saved in the address register.
| Term | Condition |
|------|:---------:|
| JGT  | comp > 0  |
| JEQ  | comp = 0  |
| JGE  | comp >= 0 |
| JLT  | comp < 0  |
| JNE  | comp != 0 |
| JLE  | comp <= 0 |
| JMP  | unconditional |

### Predefined Symbols

|Label   | RAM Address | Hex Address | Use         |
|:------|:-----------:|:-----------:|:------------|
| SP     | 0           | 0x0000      | Stack pointer. Holds the register top  of the stack in memory |
| LCL    | 1           | 0x0001      | Points to where local variables begin being stored      |
| ARG    | 2           | 0x0002      | Points to where argument variables are being stored      |
| THIS   | 3           | 0x0004      | Free register reserved for the VM      |
| THAT   | 4           | 0x0005      | Free register reserved for the VM      |
| R0-R15 | 0-16        | 0x0000-F    | Predefined short cuts to the first 15 registers in memory for ease of access. The 0-4 are already defined, 5-12 are the temp segment, and 13-15 are reserved for VM translation. |
| SCREEN | 16384       | 0x4000      | Points the base address of the screen memory map |
| KBD    | 24576       | 0x6000      | Points the base address of the keyboard memory map |

### Comments
You may comment comp lines using the // convention.

