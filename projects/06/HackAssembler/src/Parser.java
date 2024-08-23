import java.io.File;
import java.io.FileNotFoundException;
import java.util.ArrayList;
import java.util.Scanner;


public class Parser {
    private final ArrayList<String> tempFile = new ArrayList<>();
    private int position;
    private String currentLine;
    private instructionType type;

    public Parser(String fileName) throws FileNotFoundException{
        File file =  new File(fileName);
        Scanner scanner = new Scanner(file);
        position = 0;
        while(scanner.hasNextLine()){
            currentLine = scanner.nextLine().toUpperCase();
            currentLine = currentLine.replaceAll("[\\s\\t]+", ""); //trims tabs and spaces
            //NOTE: because spaces are removed labels ambiguated only by spaces will collide

            if(!currentLine.isEmpty() && !currentLine.startsWith("//")){ //removes blank lines and comments

                if(currentLine.contains("//")){
                    currentLine = currentLine.substring(0, currentLine.indexOf('/'));
                }
                tempFile.add(currentLine);
            }
        }
        scanner.close();
        //reset position so we will advance to 0 in our temp file;
        position = -1;
    }

    //made public so the assembler may access them. A little prettier than string matching
    public enum instructionType
    {
        A_COMMAND,L_COMMAND,C_COMMAND
    }

    //checks if the next line is outside of file size
    public boolean hasMoreLines() {
        if(position + 1 < tempFile.size()){
            return true;
        }
        //if there are no more lines we reset our position marker
        position = -1;
        return false;
    }

    public void advance () {
        currentLine = tempFile.get(++position);

        //we identify type each time we look at a new line and make that a property of the parser
        if(currentLine.charAt(0) == '@') {
            type = instructionType.A_COMMAND;
        } else if(currentLine.charAt(0) == '(' && currentLine.charAt(currentLine.length()-1) == ')') {
            type = instructionType.L_COMMAND;
        } else {
            //NOTE: This leaves room for errors to make their way into C instructions
            type = instructionType.C_COMMAND;
        }
    }

    public instructionType instructionType(){
        return type;
    }

    public String symbol() {
        return (type == instructionType.L_COMMAND) ? currentLine.substring(1, currentLine.length() - 1) : currentLine.substring(1);
    }

    //Returns destination string or null
    public String dest() {
        int location = currentLine.indexOf('=');

        if(location == -1){
            return null;
        }
        return currentLine.substring(0, location);
    }

    //Returns our computation code
    public String comp() {
        int beginLocation = currentLine.indexOf('=');
        beginLocation = (beginLocation == -1) ? 0 : beginLocation + 1;//advance past '='

        int endLocation = currentLine.indexOf(';');
        endLocation = (endLocation == -1) ? currentLine.length() : endLocation;

        return currentLine.substring(beginLocation, endLocation);
    }

    //returns a 3 letter jump code or null
    public String jump() {
        int location = currentLine.indexOf(';');

        if(location == -1) {
            return null;
        }
        return currentLine.substring(location+1);

    }
}
