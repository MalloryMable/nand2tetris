import java.io.File;
import java.io.FileNotFoundException;
import java.util.Scanner;


public class Parser {
    private Instruction current;
    private final Instruction head;

    public Parser(String fileName) throws FileNotFoundException{
        File file =  new File(fileName);
        try (Scanner scanner = new Scanner(file)) {
            head = new Instruction("", null);
            current = head;
           
            while(scanner.hasNextLine()){
                String currentLine;
                instructionType type;
                
                currentLine = scanner.nextLine().toUpperCase();
                currentLine = currentLine.replaceAll("[\\s\\t]+", ""); //trims tabs and spaces
                //NOTE: because spaces are removed labels ambiguated only by spaces will collide
                
                if(!currentLine.isEmpty() && !currentLine.startsWith("//")){ //removes blank lines and comments
                    if(currentLine.contains("//")){
                        currentLine = currentLine.substring(0, currentLine.indexOf('/'));
                    }
                    
                    if(currentLine.charAt(0) == '@') {
                        type = instructionType.A_COMMAND;
                    } else if(currentLine.charAt(0) == '(' && currentLine.charAt(currentLine.length()-1) == ')') {
                        type = instructionType.L_COMMAND;
                    } else {
                        //NOTE: This leaves room for errors to make their way into C instructions
                        type = instructionType.C_COMMAND;
                    }
                   
                    current.next = new Instruction( currentLine, type );
                    current = current.next;
                }
            }

            current = head.next;
        }
    }

    //made public so the assembler may access them. A little prettier than string matching
    public enum instructionType
    {
        A_COMMAND,L_COMMAND,C_COMMAND
    }

    //checks if the next line is outside of file size
    public boolean hasMoreLines() {
        if(current.next != null){
            return true;
        }
        //if there are no more lines we reset our iterator
        current = head.next;
        return false;
    }

    // this is unsafe as we use the main loop to do this check and want to avoid redundancy
    public void advance () {
        current = current.next;
    }

    public instructionType instructionType(){
        return current.type;
    }

    public String symbol() {
        return (current.type == instructionType.L_COMMAND) ? current.line.substring(1, current.line.length() - 1) : current.line.substring(1);
    }

    //Returns destination string or null
    public String dest() {
        int location = current.line.indexOf('=');

        if(location == -1){
            return null;
        }
        return current.line.substring(0, location);
    }

    //Returns our computation code
    public String comp() {
        int beginLocation = current.line.indexOf('=');
        beginLocation = (beginLocation == -1) ? 0 : beginLocation + 1;//advance past '='

        int endLocation = current.line.indexOf(';');
        endLocation = (endLocation == -1) ? current.line.length() : endLocation;

        return current.line.substring(beginLocation, endLocation);
    }

    //returns a 3 letter jump code or null
    public String jump() {
        int location = current.line.indexOf(';');

        if(location == -1) {
            return null;
        }
        return current.line.substring(location + 1);

    }

    private static class Instruction {
        String line;
        instructionType type;
        Instruction next;
        
        private Instruction(String line , instructionType type){
            this.line = line;
            this.type = type;
            next = null;
        }
    }
}
