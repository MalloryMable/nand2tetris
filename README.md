# nand2tetris

My take on nand2tetris(an online project that allows you to both define a hardware architecture and build a software architecture on top of it).

## Included Software

The nand2tetris project includes a series of tools for building(fake binaries(they are text files of zeros and ones rather than actual binaries)) from the high level JACK language. Mine are implemented in Java and once compiled can be run using the following commands.

### Assembler
Translates a .asm assembly file to the .hack binary format.

`HackAssembler [FILE]`

### VM Translator
Translates all .vm files in a taget directory to a .asm file.

`VMtranslator [FILE]`

### Compiler
Compiles a .jack project down to a .vm virtual machine language project.

`JackCompiler [FILE]`


### Shell Script
Also included is a shell script used to chain the three stages of the compiler together.

`./compile2bin.sh [FILE]`


## Part One: Hardware

The first part of the project consists of 6 chapters defining the hardware for the HACK computer.

[ 1. Basic Gates ](projects/01/README.md)\
[ 2. Arithmetic Chips ](projects/02/README.md)\
[ 3. Memory ](projects/03/README.md)\
[ 4. HACK Computer Architecture ](projects/05/README.md)\
[ 5. Assembler ](projects/06/README.md)

## Part Two: Software Stack

Below is the implementation for the JACK compiler and a JACK based OS for that can be run on the HACK Compiler.

[ 1. VM Translator ](projects/08/README.md)\
[ 2. JACK Compiler ](projects/11/README.md)\
[ 3. HACK OS ](projects/12/README.md)


## Additional Resources

Supplementary documentation

[ 1. Assembly Syntax ](projects/06/AssemblySyntax.md)\
[ 2. nand2tetris Book ](nand2tetris.pdf)