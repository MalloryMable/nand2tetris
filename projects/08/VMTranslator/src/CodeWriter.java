import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.HashMap;

public class CodeWriter {
    private final PrintWriter printWriter;
    private final HashMap<String, Integer> functionTable = new HashMap<>();
    private int ifCounter = 0;
    private String currentFile;
    private String currentFunction;

    private final static String[] predefinedSymbols = {"@LCL", "@ARG", "@THIS", "@THAT"};

    // opens a new file to write to
    public CodeWriter(String fileName) throws IOException {
        printWriter = new PrintWriter(new FileWriter(fileName+".asm"));

    }

    // sets the current file when a new file begins being parsed in main
    public void setCurrentFile(String file){
        this.currentFile = file;
        currentFunction = currentFile;
    }

    // writes the assembly for an arithmetic command to file
    public void writeArithmetic(String command){
        atTopOfStackFromPointer(); // moves to filled memory slot
        switch (command) {
            case ("NOT") -> printWriter.println("M=!M"); // acts on a slots memory
            case ("NEG") -> printWriter.println("M=-M");
            default -> {
                dataFromMemory(); // saves acted upon memory to register
                atStackPointer();
                printWriter.println("M=M-1"); // moves pointer back(pops)
                atTopOfStack(); // moves back to the top of the used stack
                switch (command) {
                    case ("ADD") -> printWriter.println("M=M+D");
                    case ("SUB") -> printWriter.println("M=M-D");
                    case ("AND") -> printWriter.println("M=M&D");
                    case ("OR") -> printWriter.println("M=M|D");
                    default -> {
                        printWriter.println("D=M-D");// Comparisons always do subtraction
                        printWriter.println(fullAtLabel("false"+ifCounter));
                        switch (command) {
                            case ("EQ") -> printWriter.println("D;JEQ");
                            case ("GT") -> printWriter.println("D;JGT");
                            case ("LT") -> printWriter.println("D;JLT");
                        }
                        // NOTE: these labels don't come from a vm file so will never collide in name space
                        printWriter.println("D=0"); // true
                        printWriter.println(fullAtLabel("end" + ifCounter));
                        unconditionalJump(); // skip
                        writeLabel("false"+ifCounter); // increments counter on last call
                        printWriter.println("D=-1"); // false
                        writeLabel("end"+ifCounter++); // used to skip setting D to false, NOTE: if incrementation
                        atTopOfStackFromPointer(); // Move to top of stack
                        memoryFromData();
                    }
                }
            }
        }
    }

    // writes assembly to interact with the stack and an indicated segment in memory
    public void writePushPop(Main.commandType type, String segment, int offset) {
        boolean memorySave = true; // As opposed to address save
        /* The added memory usage and time at compile
         * is better than extra lines of assembly
         * only flags constants as false*/

        switch (segment.toUpperCase()) {

            case "CONSTANT" -> {
                memorySave = false; // a special control flow for pushing constants
                printWriter.println("@" + offset);
            }

            case "POINTER" -> printWriter.println('@' + (offset + 3)); // Either This or That
            case "TEMP" -> printWriter.println('@' + (offset + 5)); // Base register of temp
            case "STATIC" -> printWriter.printf("@%s.%d\n", currentFile, offset); // These are variable names, offset == variable number

            case "LOCAL" -> predefinedPointer(predefinedSymbols[0], offset);
            case "ARGUMENT" -> predefinedPointer(predefinedSymbols[1], offset);
            case "THIS" -> predefinedPointer(predefinedSymbols[2], offset);
            case "THAT" -> predefinedPointer(predefinedSymbols[3], offset);
        }

        if(type == Main.commandType.C_POP) {
            dataFromAddress(); // Saves target address
            atTempVar(); // free register here used to augment data
            memoryFromData(); // stores target

            followDecrementedStackPointer();// moves to filled slot and alters pointer
            dataFromMemory(); // holds found value in register
            atTempVar();
            followPointer(); // moves to stored target
            memoryFromData(); // saves to targeted register
        } else if(type == Main.commandType.C_PUSH){
            if(memorySave){
                dataFromMemory(); // saves value found in a given segment
            }else{
                dataFromAddress(); // for constants
            }

            followStackPointer();
            memoryFromData(); // assigns pulled memory
            atStackPointer();
            printWriter.println("M=M+1"); // advances to a new Empty Space
        }
    }

    // Initalization code calls Sys init() as defined by the compiler
    public void writeInit() {
        // Saves 256 
        printWriter.println("@256");
        dataFromAddress();
        atStackPointer();
        memoryFromData();
        // Calls Sys.init
        writeCall("Sys.init", 0);
    }

    // writes a valid assembly label(it's fine that it's all upper case and looks funky)
    public void writeLabel(String label) {
        writeSimpleLabel(String.format("%s$%s", currentFunction, label));
    }

    // writes the assembly to go to a given label
    public void writeGoto(String label) {
        printWriter.println(fullAtLabel(label));
        unconditionalJump();
    }

    // writes an assembly if statement
    public void writeIf(String label) {
        followDecrementedStackPointer();
        dataFromMemory();
        printWriter.println(fullAtLabel(label));
        printWriter.println("D;JEQ");
    }

    // writes a function call
    public void writeCall(String function, int varCount) {
        // checks if the called functionTable has a period and is pointing to another file
        String calledFunction = function.contains(".") ?
            function : String.format("%s.%s", currentFile, function);
        
        // checks if a function exists
        functionExists(function);

        // creates a unique return label
        int timesCalled = functionTable.get(calledFunction) + 1;
        String returnLabel = calledFunction + '$' + "return" + timesCalled;
        
        // updates the number of times the function has been called
        functionTable.put(calledFunction, timesCalled);

        // push return-address
        printWriter.println('@' + returnLabel );
        dataFromAddress();
        followStackPointer();
        memoryFromData();

        // pushes the pointer in LCL, ARG, THIS, and THAT to stack
        for (String symbol: predefinedSymbols){
            printWriter.println(symbol);
            dataFromMemory();
            followIncrementedStackPointer();
            memoryFromData();
        }

        atStackPointer();
        printWriter.println("MD=M+1");// saves new empty Stack Pointer to data
        atLocal(); // moves to local
        memoryFromData(); // saves the pointer to register

        printWriter.println('@' + (5 + varCount));
        printWriter.println("A=D-A");
        printWriter.println(predefinedSymbols[1]); // 
        memoryFromData();

        // we cannot use traditional goTo because of the unconventional at
        printWriter.println('@'+ calledFunction);
        unconditionalJump();

        writeSimpleLabel(returnLabel);
    }

    // writes the assembly to define a function to file
    public void writeFunction(String function, int varCount){     
        currentFunction = String.format("%s.%s", currentFile, function);
        functionExists(currentFunction);
       

        writeSimpleLabel(currentFunction);
        followStackPointer();
        for(int m = 0; m < varCount; m++){
            printWriter.println("M=0"); // saves empty values
            followIncrementedStackPointer(); // one needless address write
        }
    }

    // assembly to construct a valid return statement
    public void writeReturn(){
        // Frame = LCL
        atLocal();
        dataFromMemory();
        atFrame();
        memoryFromData();

        // RET = *(FRAME - 5)
        printWriter.println("@5");
        printWriter.println("A=D-A"); // FRAME is still stored in Data
        followPointer();
        dataFromMemory(); // D = *FRAME-5)

        atTempVar(); // RET
        memoryFromData();
        followDecrementedStackPointer();
        dataFromMemory();

        atArgument();
        followPointer();
        memoryFromData();

        atArgument();
        printWriter.println("D=M+1");

        atStackPointer();
        memoryFromData();

        for(int i = 3; i >= 0; i--){
            atFrame();
            followDecrementedPointer();
            dataFromMemory();
            printWriter.println(predefinedSymbols[i]); // moves through list backwards
            memoryFromData();
        }

        atTempVar();
        followPointer();
        unconditionalJump();
    }

    // closes file from main since when a given file finishes being written is unknown
    public void close(){
        printWriter.close();
    }

    // PRIVATE Functions

    // checks if this function has been called or declared yet
    private void functionExists(String function) {
        if(!functionTable.containsKey(function)){
            functionTable.put(function, 0);
        } 
    }

    // writes a return or at label that matches the conventions elsewhere
    private String fullAtLabel(String label){
        return '@' + currentFunction +'$' + label;
    }

    // a version of label writing for labels that use an internal convention
    private void writeSimpleLabel(String label) {
        printWriter.println('(' + label +')');
    }

    // A pointer accesses the memory
    private void predefinedPointer(String segment, int offset) {
        printWriter.println(segment); // gets the predefined segment name
        if(offset != 0){
            dataFromMemory(); // stores pointer base location
            printWriter.println('@' + offset);
            printWriter.println("A=D+A"); // moves to pointed to address plus offset
        } else {
            dataFromAddress(); // moves directly to pointed to address
        }
    }

    // Goes to the stack pointer and follows it
    private void followStackPointer(){
        atStackPointer(); // @SP
        followPointer();  // A=M
    }

    // sets th address of the stack pointer to one greater than it currently is and moves there
    private void followIncrementedStackPointer(){
        atStackPointer();
        printWriter.println("AM=M+1");
    }

    // sets th address of the stack pointer to one greater than it currently is and moves there
    private void followDecrementedStackPointer() {
        atStackPointer();
        followDecrementedPointer();
    }
 
    // moves to one below the current stack pointer without
    private void atTopOfStackFromPointer(){
        atStackPointer(); // @SP
        atTopOfStack(); // A=M-1
    }

    private void atStackPointer() { printWriter.println("@SP");} // moves to RAM[0], the stack pointer

    private void atLocal() { printWriter.println(predefinedSymbols[0]);} // @LCL

    private void atArgument() { printWriter.println(predefinedSymbols[1]);} // @ARG

    private void atTempVar() { printWriter.println("@R15");} // hard coded temp variable location

    private void atFrame() { printWriter.println("@R14");} // hard coded from variable location

    private void unconditionalJump() { printWriter.println("0;JMP");} // jumps to the above address

    private void dataFromMemory() { printWriter.println("D=M");} // saves memory to register

    private void dataFromAddress() { printWriter.println("D=A");} // saves address to register

    private void memoryFromData() {printWriter.println("M=D");} // saves register value to current memory slot

    private void followPointer() {printWriter.println("A=M");} // set address to current memory slot

    private void atTopOfStack() {printWriter.println("A=M-1");} // moves to one less than the pointer stored in memory

    private void followDecrementedPointer(){ printWriter.println("AM=M-1");} // sets and moves to given address minus one
}
