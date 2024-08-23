import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.ArrayList;

/*
 *  Made the call to hard code the flow of the way this writes rather than breaking out string pointer
 *  movement because while usages are similar. Very few are identical and breaking them out takes about
 *  the same number of lines and makes this whole program less readable
*/

public class CodeWriter {
    private final PrintWriter printWriter;
    private final ArrayList<String> functions = new ArrayList<>();
    private final ArrayList<Integer> functionCallCount = new ArrayList<>();
    private int ifCounter = 0;
    private String currentFile;
    private String currentFunction;



    private final static String[] predefinedSymbols = {"@LCL", "@ARG", "@THIS", "@THAT"};

    // opens a new file to write to
    public CodeWriter(String fileName) throws IOException {
        printWriter = new PrintWriter(new FileWriter(fileName));
    }


    //writes the assembly for an arithmetic command to file
    public void writeArithmetic(String command){
        atTopOfStackFromPointer(); //moves to filled memory slot
        switch (command) {
            case ("NOT") -> printWriter.println("M=!M"); //acts on a slots memory
            case ("NEG") -> printWriter.println("M=-M");
            default -> {
                dataFromMemory(); // saves acted upon memory to register
                atStackPointer();
                printWriter.println("M=M-1"); //moves pointer back(pops)
                atTopOfStack(); //moves back to the top of the used stack
                switch (command) {
                    case ("ADD") -> printWriter.println("M=M+D");
                    case ("SUB") -> printWriter.println("M=M-D");
                    case ("AND") -> printWriter.println("M=M&D");
                    case ("OR") -> printWriter.println("M=M|D");
                    default -> {
                        printWriter.println("D=M-D");//Comparisons always do subtraction
                        printWriter.println(fullAtLabel("false"+ifCounter));
                        switch (command) {
                            case ("EQ") -> printWriter.println("D;JEQ");
                            case ("GT") -> printWriter.println("D;JGT");
                            case ("LT") -> printWriter.println("D;JLT");
                        }
                        //NOTE: these labels don't come from a vm file so will never collide in name space
                        printWriter.println("D=0"); // true
                        printWriter.println(fullAtLabel("end" + ifCounter));
                        unconditionalJump(); //skip
                        writeLabel("$false"+ifCounter); // increments counter on last call
                        printWriter.println("D=-1"); // false
                        writeLabel("$end"+ifCounter++); // used to skip setting D to false, NOTE: if incrementation
                        atTopOfStackFromPointer(); // Move to top of stack
                        memoryToData();
                    }
                }
            }
        }
    }

    //writes a valid assembly label(it's fine that it's all upper case and looks funky)
    public void writeLabel(String label) {printWriter.printf("(%s)\n", label);}

    //writes the assembly to go to a given label
    public void writeGoto(String label) {
        printWriter.println(fullAtLabel(label));
        unconditionalJump();
    }

    //writes an assembly if statement
    public void writeIf(String label) {
        followDecrementedStackPointer();
        dataFromMemory();
        printWriter.println(fullAtLabel(label));
        printWriter.println("D;JEQ");
    }

    // writes assembly to interact with the stack and an indicated segment in memory
    public void writePushPop(Main.commandType type, String segment, int offset) {
        boolean memorySave = true; //As opposed to address save
        /*The added memory usage and time at compile
         *is better than extra lines of assembly
         *only flags constants as false*/

        //TODO: Offset may need to be corrected to be above predefined addresses
        switch (segment.toUpperCase()) {

            case "CONSTANT" -> {
                memorySave = false; //a special control flow for pushing constants
                printWriter.println("@" + offset);
            }

            case "POINTER" -> printWriter.println("@" + (offset + 3));
            case "TEMP" -> printWriter.println("@" + (offset + 5));
            case "STATIC" -> printWriter.printf("@%s.%d\n", currentFile, offset); //may need to write static labels somewhere

            case "LOCAL" -> predefinedPointer(predefinedSymbols[0], offset);
            case "ARGUMENT" -> predefinedPointer(predefinedSymbols[1], offset);
            case "THIS" -> predefinedPointer(predefinedSymbols[2], offset);
            case "THAT" -> predefinedPointer(predefinedSymbols[3], offset);
        }

        if(type == Main.commandType.C_POP) {
            dataFromAddress(); //Saves target address
            atTempVar(); //final temp ram slot
            memoryToData(); //stores target

            followDecrementedStackPointer();//moves to filled slot and alters pointer
            dataFromMemory(); //holds found value in register
            atTempVar();
            followPointer(); //moves to stored target
            memoryToData(); //saves to targeted register
        } else if(type == Main.commandType.C_PUSH){
            if(memorySave){
                dataFromMemory(); //saves value found in a given segment
            }else{
                dataFromAddress(); //for constants
            }

            followStackPointer();
            memoryToData(); //assigns pulled memory
            atStackPointer();
            printWriter.println("M=M+1"); //advances to a new Empty Space
        }
    }

    //writes a function call
    public void writeCall(String function, int varCount) {
        currentFunction = function;
        functionCheck(function);

        //this is done because finding the index twice is less efficient than saving it for a moment
        int index = (functions.indexOf(currentFunction));

        functionCallCount.set(index, 1+functionCallCount.get(index));
        printWriter.println("@" + (String.valueOf(functionCallCount.get(index))));
        dataFromAddress();
        followStackPointer();
        memoryToData();

        for (String symbol: predefinedSymbols){
            printWriter.println(symbol);
            dataFromMemory();
            followIncrementedStackPointer();
            memoryToData();
        }

        atStackPointer();
        printWriter.println("MD=M+1");//saves new empty Stack Pointer to data
        atLocal(); //moves to local
        memoryToData(); //saves the pointer to register

        printWriter.println("@" + (5 + varCount));
        printWriter.println("A=D-A");
        printWriter.println(predefinedSymbols[1]);
        memoryToData();

        //we cannot use traditional goTo because of the unconventional at
        printWriter.println("@"+ currentFunction);
        unconditionalJump();
    }

    //writes the assembly to define aa function to file
    public void writeFunction(String function, int varCount){
        functionCheck(function);
        writeLabel(function);
        followStackPointer();
        for(int m = 0; m < varCount; m++){
            printWriter.println("M=0"); //saves empty values
            followIncrementedStackPointer(); //one needless address write
        }
    }

    //assembly to construct a valid return statement
    public void writeReturn(){
        atLocal();
        dataFromMemory();
        atFrame();
        memoryToData();

        printWriter.println("@5");
        printWriter.println("A=D-A");
        followPointer();
        dataFromMemory();

        atTempVar();
        memoryToData();
        followDecrementedStackPointer();
        dataFromMemory();

        atArgument();
        followPointer();
        memoryToData();

        atArgument();
        printWriter.println("D=M+1");

        atStackPointer();
        memoryToData();

        for(int i = 3; i >= 0; i--){
            atFrame();
            followDecrementedPointer();
            dataFromMemory();
            printWriter.println(predefinedSymbols[i]); //moves through list backwards
            memoryToData();
        }

        atTempVar();
        followPointer();
        unconditionalJump();
    }

    //closes file from main since when a given file finishes being written is unknown
    public void close(){
        printWriter.close();
    }

    //checks if this function has been called or declared yet
    private void functionCheck(String function) {
        if(!functions.contains(function)){
            functions.add(function);
            functionCallCount.add(0);
        }

    }

    //sets the current file when a new file begins being parsed in main
    public void setCurrentFile(String file){this.currentFile = file;}

    //writes a return or at label that matches the conventions elsewhere
    private String fullAtLabel(String label){
        return "@" + currentFunction +"$" + label;
    }
    //A pointer accesses the memory


    private void predefinedPointer(String segment, int offset) {
        printWriter.println(segment); //gets the predefined segment name
        if(offset != 0){
            dataFromMemory(); //stores pointer base location
            printWriter.println("@" + offset);
            printWriter.println("A=D+A"); //moves to pointed to address plus offset
        } else {
            dataFromAddress(); //moves directly to pointed to address
        }
    }

    //Goes to the stack pointer and follows it
    private void followStackPointer(){
        atStackPointer(); // @SP
        followPointer();  // A=M
    }

    //sets th address of the stack pointer to one greater than it currently is and moves there
    private void followIncrementedStackPointer(){
        atStackPointer();
        printWriter.println("AM=M+1");
    }

    //sets th address of the stack pointer to one greater than it currently is and moves there
    private void followDecrementedStackPointer() {
        atStackPointer();
        followDecrementedPointer();
    }

    //moves to one below the current stack pointer without
    private void atTopOfStackFromPointer(){
        atStackPointer(); // @SP
        atTopOfStack(); // A=M-1
    }

    private void atStackPointer() { printWriter.println("@SP");} // moves to RAM[0], the stack pointer

    private void atLocal() { printWriter.println(predefinedSymbols[0]);} // @LCL

    private void atArgument() { printWriter.println(predefinedSymbols[1]);} // @ARG

    private void atTempVar() { printWriter.println("@15");} //hard coded temp variable location

    private void atFrame() { printWriter.println("@14");} //hard coded from variable location

    private void unconditionalJump() { printWriter.println("0;JMP");} //jumps to the above address

    private void dataFromMemory() { printWriter.println("D=M");} // saves memory to register

    private void dataFromAddress() { printWriter.println("D=A");} // saves address to register

    private void memoryToData() {printWriter.println("M=D");} // saves register value to current memory slot

    private void followPointer() {printWriter.println("A=M");} // set address to current memory slot

    private void atTopOfStack() {printWriter.println("A=M-1");} // moves to one less than the pointer stored in memory

    private void followDecrementedPointer(){ printWriter.println("AM=M-1");} //sets and moves to given address minus one
}
