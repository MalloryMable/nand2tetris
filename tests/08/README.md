## VM Translator

`VMTranslator [FILE | DIR]`

A fully featured VM Translator. Takes one file or directory as an argument, and translates each .vm file into a unified .asm file.

### Parser
**Parser(String)**\
Sanatizes incoming data and saves each line as a linked list.

**boolean hasMoreLines()**\
checks if there are any more valid lines of code

**advance()**\
Moves the current line being examined by the parser, splits the line into constituent parts, and finds and sets the type of the current command.

**commandType commandType()**\
Getter method for the current lines command type.

**String arg1()**\
Returns the first argument of the current line.

**int arg2()**\
Returns the second argument of the current line.

### CodeWriter
Translates abstract VM commands into their asm equivalents 

**CodeWriter(String)**\
Takes the name of our assembly code, opens a file with this name, and calls `writeInit()`.

**void fileName(String)**\
Updates the name of the file being parsed to avoid name space collision for labels.

**void writeArithmetic(String)**\
Acts on the top integer saved at the register referenced by the top of the stack, and(if not single argument operations like not and negation) next int saved to the stack.
If this comparison is part of a jump an if else statement is saved as well using these pieces of data.

**void writePushPop(commandType, String, int)**\
Takes the name of a segment and an amount we offset from the base of that memory segment.
In the push case the memory stored at the pointer found by taking the offset of the given segment is pushed onto the stack.
In the case of pop the top item of the stack is removed, saved in the given position in the segment and the stack is increased in size by one.


**void writeInit()**\
Writes the bootstrap code.
This involves setting the stackpointer to 256 and calling `Sys.init()` passed down by the compiler.

**void writeLable(String)**\
Takes a labels name, appends it the label to the function and file name (`[file].[function]$label`)

**void writeGoTo(String)**\
Writes an unconditional jump to a given label.

**void writeIf(String)**\
Takes the memory stored in the top register of the stack.
If the top register is equal to zero then a jump is carried out to the label passed in by the argument.

**void writeCall(String, int)**\
Writes the assembly code for calling a given command. Takes the number of vars used.
Pushes the pointers to all local variables and a return address pointer to memory onto the stack as a Frame.
Then jumps to the label for the current function and begins executing.

**void writeFunction(String, int)**\
Writes a label for the new function and begins the new function by allocating empty local variables for all args needed(as denoted by the int passed as an argument).

**void writeReturn()**\
Moves all of the stored variables out of the frame before following the return address back to the point in memory being executed from previously.

**void close()**\
Closes output file opened by the constructor.

### Main
Makes a parser that is able to pass information that may then be used by the CodeWriter.
