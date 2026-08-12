import java.io.FileNotFoundException;
import java.io.IOException;
import java.nio.file.DirectoryIteratorException;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.Path;


public class VMTranslator {
    
    public enum commandType {
        C_ARITHMETIC, C_PUSH, C_POP , C_LABEL, C_GOTO, C_IF, C_FUNCTION, C_RETURN, C_CALL
    }
    static CodeWriter writer;
    public static void main(String[] args) throws IOException {

        Path path = Path.of(args[0]);
        Path targetDirectory = (args.length == 2)?
                Path.of(args[1]): path;

        //checks if the user input is a directory or single file and uses the appropriate logic
        if((Files.isDirectory(path))){
            writer = new CodeWriter(targetDirectory + "/" + path.getFileName());
            writer.writeInit();
            //loop through valid vm files
            openDir(path);
        } else {
            //rather than creating the filename string twice to trim the file type a variable is created
            String filename = path.getFileName().toString();
            String directoryPath = targetDirectory.toString();
            if(directoryPath.contains(".")){
                directoryPath = directoryPath.substring(0, directoryPath.lastIndexOf('/') -1);
            }
            writer = new CodeWriter( directoryPath + filename.substring(0,filename.lastIndexOf('.')));
            writer.writeInit();
            writeToFile(path.toString());
        }

        writer.close();
    }

    //opens every folder and reads in all valid files
    private static void openDir(Path dir) throws IOException{
        try (DirectoryStream<Path> stream = Files.newDirectoryStream(dir)) {
            for (Path file: stream) {
                String stringFile = file.getFileName().toString();
                // recursion
                if(Files.isDirectory(file)){
                    openDir(file);
                // base case: ensures the file type is vm, and then that we aren't compiling Sys.vm early
                } else if(stringFile.substring(stringFile.lastIndexOf('.')).equals(".vm")
                        && ! stringFile.equals("Sys.vm")) {
                    writeToFile(file.toString());
                }
            }
        } catch (IOException | DirectoryIteratorException x) {
            System.err.println(x);
        }
    }

    //writes a given file to the created file
    private static void writeToFile(String inFile) throws FileNotFoundException {
        writer.setCurrentFile(inFile.substring(1+inFile.lastIndexOf('\\'), inFile.lastIndexOf('.')).toUpperCase());

        Parser parser = new Parser(inFile);

        //goes through parsed .vm file writing assembly to a new .asm file
        while(parser.hasMoreLines()){
            parser.advance();
            if(null != parser.commandType()) //You can't change it to a switch statement and make the enumeration globally available
            switch (parser.commandType()) {
                case C_PUSH, C_POP -> writer.writePushPop(parser.commandType(), parser.arg1(), parser.arg2());
                case C_ARITHMETIC -> writer.writeArithmetic(parser.arg1());
                case C_LABEL -> writer.writeLabel(parser.arg1());
                case C_GOTO -> writer.writeGoto(parser.arg1());
                case C_IF -> writer.writeIf(parser.arg1());
                case C_CALL -> writer.writeCall(parser.arg1(), parser.arg2());
                case C_FUNCTION -> writer.writeFunction(parser.arg1(), parser.arg2());
                case C_RETURN -> writer.writeReturn();
                default -> {
                    // NOTE by returning the line from parser we could get better error handling
                    throw new RuntimeException("Invalid command type found");
                }
            }
        }
    }
}
