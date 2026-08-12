## Assembler
Translates a .asm assembly file to the .hack binary format.

`HackAssembler [FILE]`


### Parser
Reads in an assembly file, sanitizes it, and tacks command types as Address, Label, or Computation.
This module also contains the InstructionType enumeration used for denoting what type of instruction the current line is.

**Parser(String)**\
Opens the passed file(this secretly uses a path, please don't tell the MIT professors), and makes a first pass removing unused lines, and spaces.

**boolean hasMoreLines()**\
Checks if there is a line remaining to be parsed.

**void advance()**\
Moves on to the next line, and if address or label syntax is detected makes that the parsers label type, otherwise the type defaults to comp.

**instructionType instructionType()**\
Getter function for the type of the current instruction.

**String symbol()**\
Returns the symbol contained within an address or label instruction.

**String dest()**\
Returns the destination of a given comp code as a String.

**String comp()**\
Returns a string containing only the computation the ALU will be undertaking.

**String jump()**\
Returns the 3 character jump code or null if no jump condition was included.

### Code
This module is used to translate the substrings of comp collected by the parser into strings of 1s and 0s.

**String comp(String)**\
Translates a string to the 7 bits used to specifiy ALU behavior.

**String dest(String)**\
Translates a string to the 3 bits indicating where the ALU output will be sent. Note that these may be passed in any order.

**String jump(String)**\
Translates a string to the 3 bits specifying the jump conditions.

### SymbolTable
A hashmap containing the register each symbol corresponds to.

**SymbolTable()**\
Initalizes the table, adding predefined symbols.

**void addEntry(String, int)**\
Adds a new symbol to the table.

**boolean contains(String)**\
Checks if a given string is contained within the symbol table.

**int getAddress(String)**\
Returns the address of a given symbol.
Only meant to be used after contains has been run, so there is no built in error checking.

### Main
Parses the asm file in three passes. Pass one is handled by the parser and does sanitization and standardization.
The second pass finds lines where labels are declared and removes them from the code storing the line they appear on. 
This allows labels to be referenced before they are declared safely.
This is also why the parser advance function resets the counter when reaching the end of the stored lines.
The third pass handles both other instruction types and writes the result of translating parsed code snippets to a new file with the same name as the file passed in. 



NOTES TO MALLORY:
Maybe add the option to name an output file/directory
