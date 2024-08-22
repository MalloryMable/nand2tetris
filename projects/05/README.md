## Computer Hardware Architecture
The Hack computer consists of 32K registers of ROM for holding the OS written in chapter 12(what's a BIOS?), the CPU, and 2 16K RAM sticks.

Since the CPU determines the structure of opcodes the syntax of opcode is as follows:
Address: 0vvvvvvvvvvvvvvv
Instruction: 111accccccdddjjj
a = Determines if the ALU uses the value of the register or value stored within a register.
c = In order the 6 bits for modifying operations of the ALU(zx, nx, zy, ny, f, no).
d = specifies where to the result of the ALU to(data register(D), address register(A), Program Counter register(PC)).
j = specifies a condition for jumping after instruction executes(in order: less than zero, equal to zero, greater than zero).

### CPU
**Input**
Takes in a the value stored in the current register as a 16 bit value, a 16 bit instruction code, and a single bit indicating whether to flush memory and move to the begnining of the current program.

**Output**
Sends a 16 bit output value, a single bit instructing whether to write to current register, the address of the to be targeted in the next loop, and the 15 bit value to load into the incrementer to determine what instruction to load next.

**Implementation**
The CPU is a wrapper combining the ALU and PC(mostly consisting of Mux chips). As well as a special 16 bit register used for storing output from the ALU for convient math, and a special register for storing one RAM address.
Notice that the value being incremented by the PC can be altered by the jump condition being met, or the reset bit being sent into the CPU. Further it is relevant that the reset bit supercedes jumps.

### Memory
15 bits of potential address space consisting of a RAM16K stick(0x0000-0x3FFF), a screen map(using a builtin chip for now)(0x4000-0x5FFF), and a keyboard chip dedicated to mapping inputs to ASCII values(builtin chip)(0x6000-0x6098). The remaining potential addresses are left unused.