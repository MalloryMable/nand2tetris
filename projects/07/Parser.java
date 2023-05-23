import java.io.File;
import java.io.FileNotFoundException;
import java.util.ArrayList;
import java.util.Scanner;


public class Parser{
    private String[] LineArray = new String[3];
    private final ArrayList<String> tempFile = new ArrayList<>();
    private int position = -1; //initialized so that initial advance() moves to position: 0
    private Main.commandType type;

    //Opens file and reads in all valid lines of code to Temp File
    public Parser(String fileName) throws FileNotFoundException {
        File file = new File(fileName);
        Scanner scanner = new Scanner(file);
        while (scanner.hasNextLine()) {
            String currentLine = scanner.nextLine()
                    .toUpperCase() //normalizes all text to uppercase
                    .replaceAll("\\t+", "") //removes all tabs
                    .replaceAll("\\s+", " ") //trims sets of spaces to one space
                    .replaceAll("^\\s", ""); //removes leading spaces

            if (!currentLine.isEmpty() && !currentLine.startsWith("//")) { //removes blank lines and comments

                if (currentLine.contains("//")) {
                    currentLine = currentLine.substring(0, currentLine.indexOf('/'));
                }
                tempFile.add(currentLine);
            }
        }
    }

    //checks if the next line is outside of file size
    public boolean hasMoreLines() {
        return position + 1 < tempFile.size();
    }

    //Moves through Temp File(a list of strings)
    public void advance () {
        //we identify the first word of our instruction cutting off at the first space
        LineArray = tempFile.get(++position).split("\\s");
        type = getType(LineArray[0]);

    }

    //returns command type determined at advancement
    public Main.commandType commandType(){ return type;}

    //returns the first argument as a string
    public String arg1() {
        return (type == Main.commandType.C_ARITHMATIC)? LineArray[0]: LineArray[1]; // Arg is
    }

    //returns the second argument as int
    public int arg2() {
        return Integer.parseInt(LineArray[2]);
    }

    //Returns the command type based on the first word
    private Main.commandType getType(String firstWord) { //Only called once broken out for logical division
        return switch (firstWord) {
            case "PUSH" -> Main.commandType.C_PUSH;
            case "POP" -> Main.commandType.C_POP;
            case "ADD", "SUB", "NEG", "EQ", "GT", "LT", "AND", "OR", "NOT" -> Main.commandType.C_ARITHMATIC;
            default -> throw new IllegalStateException("Unexpected value: " + firstWord);
        };
    }
}
