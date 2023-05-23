import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;

/*
 *  Made the call to hard code the flow of the way this writes rather than breaking out string pointer
 *  movement because while usages are similar. Very few are identical and breaking them out takes about
 *  the same number of lines and makes this whole program less readable
*/

public class CodeWriter {
    private final PrintWriter printWriter;
    private int ifCounter = 0;
    private String fileName;
    // opens a new file to write to
    public CodeWriter(String fileName) throws IOException {
         printWriter = new PrintWriter(new FileWriter(fileName+".asm"));
    }

    public void setFileName(String fileName){
        this.fileName = fileName;
    }

    //writes the assembly for an arithmetic command to file
    public void writeArithmetic(String command){
        printWriter.println("@SP");
        printWriter.println("A=M-1"); //moves to filled memory slot
        switch (command) {
            case ("NOT") -> printWriter.println("M=!M"); //acts on a slots memory
            case ("NEG") -> printWriter.println("M=-M");
            default -> {
                printWriter.println("D=M"); // saves acted upon memory to register
                printWriter.println("@SP");
                printWriter.println("M=M-1"); //moves pointer back
                printWriter.println("A=M-1"); //moves back to the top of the used stack
                switch (command) {
                    case ("ADD") -> printWriter.println("M=M+D");
                    case ("SUB") -> printWriter.println("M=M-D");
                    case ("AND") -> printWriter.println("M=M&D");
                    case ("OR") -> printWriter.println("M=M|D");
                    default -> {
                        printWriter.println("D=M-D");
                        printWriter.println("@false" + ifCounter);
                        switch (command) {
                            case ("EQ") -> printWriter.println("D;JEQ");
                            case ("GT") -> printWriter.println("D;JGT");
                            case ("LT") -> printWriter.println("D;JLT");
                        }
                        printWriter.println("D=0"); // true
                        printWriter.println("@end" + ifCounter);
                        printWriter.println("0;JMP"); //skip
                        printWriter.println("(false" + ifCounter + ")"); // increments counter on last call
                        printWriter.println("D=-1"); // false
                        printWriter.println("(end" + ifCounter++ + ")"); // used to skip setting D to false, NOTE: if incrementation
                        printWriter.println("@SP");
                        printWriter.println("A=M-1"); // Move to top of stack
                        printWriter.println("M=D");
                    }
                }
            }
        }

    }

    // writes assembly to interact with the stack and an indicated segment in memory
    public void writePushPop(Main.commandType type, String segment, int offset) {
        boolean memorySave = true; //As opposed to address save
        /*The added memory usage and time at compile
         *is better than extra lines of assembly
         *only flags constants as false*/

        switch (segment) {

            case "POINTER" -> printWriter.println("@" + (offset + 3));

            case "TEMP" -> printWriter.println("@" + (offset + 5));

            case "STATIC" -> printWriter.println("@" + (offset + 16));


            case "CONSTANT" -> {
                memorySave = false;
                printWriter.println("@" + offset);
            }
            case "LOCAL" -> predefinedPointer("LCL", offset);
            case "ARGUMENT" -> predefinedPointer("ARG", offset);
            //NOTE: ARG/LCL might have a special case at 0 !
            case "THIS" -> predefinedPointer("THIS", offset);
            case "THAT" -> predefinedPointer("THAT", offset);
        }

        if(type == Main.commandType.C_POP) {
            printWriter.println("D=A"); //Saves target address
            printWriter.println("@15"); //final temp ram slot
            printWriter.println("M=D"); //stores target

            printWriter.println("@SP");
            printWriter.println("AM=M-1"); //moves to filled slot
            printWriter.println("D=M"); //holds found value in register
            printWriter.println("@15");
            printWriter.println("A=M"); //moves to stored target
            printWriter.println("M=D"); //saves to targeted register
        } else if(type == Main.commandType.C_PUSH){
            if(memorySave){
                printWriter.println("D=M"); //saves value found in a given segment
            }else{
                printWriter.println("D=A"); //for constants
            }

            printWriter.println("@SP");
            printWriter.println("A=M");
            printWriter.println("M=D"); //assigns pulled memory
            printWriter.println("@SP");
            printWriter.println("M=M+1"); //advances to a new Empty Space
        }
    }

    //closes file from main since when a given file finishes being written is unknown
    public void close(){
        printWriter.close();
    }

    //A pointer accesses the memory
    private void predefinedPointer(String segment, int offset) {
        printWriter.println("@" + segment); //gets the predefined segment name
        if(offset != 0){
            printWriter.println("D=M"); //stores pointer base location
            printWriter.println("@" + offset);
            printWriter.println("A=D+A"); //moves to pointed to address plus offset
        } else {
            printWriter.println("A=M"); //moves directly to pointed to address
        }
    }
}
