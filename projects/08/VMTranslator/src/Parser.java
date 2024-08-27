import java.io.File;
import java.io.FileNotFoundException;
import java.util.Scanner;


public class Parser{
    private String[] lineArray = new String[3];
    private Main.commandType type;
    boolean checked = false;
    boolean moreLines;
    Scanner scanner;
    String nextLine;

    //Opens file and reads in all valid lines of code to Temp File
    public Parser(String fileName) throws FileNotFoundException {

        File file = new File(fileName);
        scanner = new Scanner(file);
        
    }

    //checks if the next line is outside of file size
    public boolean hasMoreLines() {
        if (!checked) {
            checked = true;
            while (scanner.hasNextLine()) {
                String line = scanner.nextLine()
                    .replaceAll("\\t+", "") //removes all tabs
                    .replaceAll("\\s+", " ") //trims sets of spaces to one space
                    .replaceAll("^\\s", "") //removes leading space
                    .replaceAll("\\s$", ""); //removes trailing space

                if (!(line.isEmpty() || line.startsWith("//"))) { //removes blank lines and comments
                    if (line.contains("//")) {
                        line = line.substring(0, line.indexOf('/'));
                    }

                    nextLine = line;
                    moreLines = true;
                    break;
                }
            }
            moreLines = false;
        } 
        return moreLines;
    }

    //Moves through Temp File(a list of strings)
    public void advance () {
        //we identify the first word of our instruction cutting off at the first space
        lineArray = nextLine.split("\\s");
        type = getType(lineArray[0]);
        checked = false;

    }

    //returns command type determined at advancement
    public Main.commandType commandType() { return type;}

    //returns the first argument as a string
    public String arg1() {
        return (type == Main.commandType.C_ARITHMETIC)? lineArray[0]: lineArray[1];
    }

    //returns the second argument as int
    public int arg2() {
        return Integer.parseInt(lineArray[2]);
    }

    //Returns the command type based on the first word
    private Main.commandType getType(String firstWord) { //Only called once broken out for logical division
        return switch (firstWord.toUpperCase()) {
            case "PUSH" -> Main.commandType.C_PUSH;
            case "POP" -> Main.commandType.C_POP;
            case "ADD", "SUB", "NEG", "EQ", "GT", "LT", "AND", "OR", "NOT" -> Main.commandType.C_ARITHMETIC;
            case "LABEL" -> Main.commandType.C_LABEL;
            case "GOTO" -> Main.commandType.C_GOTO;
            case "IF-GOTO" -> Main.commandType.C_IF;
            case "CALL" -> Main.commandType.C_CALL;
            case "FUNCTION" -> Main.commandType.C_FUNCTION;
            case "RETURN" ->  Main.commandType.C_RETURN;
            default -> throw new IllegalStateException("Unexpected value: " + firstWord);
        };
    }
}
