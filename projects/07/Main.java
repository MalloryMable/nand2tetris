import java.io.FileNotFoundException;
import java.io.IOException;
import java.nio.file.DirectoryIteratorException;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.Path;


public class Main {

    enum commandType {
        C_ARITHMETIC, C_PUSH, C_POP , C_LABEL, C_GOTO, C_IF, C_FUNCTION, C_RETURN, C_CALL
    }
    static CodeWriter writer;
    public static void main(String[] args) throws IOException {

        Path path = Path.of(args[0]);

        //checks if the user input is a directory or single file and uses the appropriate logic
        if((Files.isDirectory(path))){
            writer = new CodeWriter( String.format("%s/%s.asm",path.toString(), path.getFileName()));
            //Sys.init
            try {
                writeToFile(getInit(path).toString());
            } catch (NullPointerException e){
                System.out.println("Warning! Missing init file");
            }
            //loop through valid vm files
            openDir(path);
        } else {
            writer = new CodeWriter( (path.toString().contains("."))? path.toString().replace(".vm", ".asm"): path + ".asm");
            writeToFile(path.toString());
        }


        writer.close();
    }

    //recursively searches for the Sys.vm file to initiate our assembly
    private static Path getInit(Path dir) throws IOException {
        try (DirectoryStream<Path> stream = Files.newDirectoryStream(dir)) {
            for (Path file: stream) {
                if(Files.isDirectory(file)){
                    return getInit(file);
                } else if(file.getFileName().toString().equals("Sys.vm")) {
                    return file;
                }
            }
        } catch (IOException | DirectoryIteratorException directoryError) {
            System.out.println("Directory error: " + directoryError);
        }
        return null;
    }

    //opens every folder and reads in all valid files
    private static void openDir(Path dir) throws IOException{
        try (DirectoryStream<Path> stream = Files.newDirectoryStream(dir)) {
            for (Path file: stream) {
                String stringFile = file.getFileName().toString();

                if(Files.isDirectory(file)){
                    openDir(file);
                } else if(stringFile.substring(stringFile.lastIndexOf('.')).equals(".vm")
                        && ! stringFile.equals("Sys.vm")) {
                    writeToFile(file.toString());
                }
            }
        } catch (IOException | DirectoryIteratorException x) {
            System.out.println("Caught error: "+ x);
        }
    }

    //writes a given file to the created file
    private static void writeToFile(String inFile) throws FileNotFoundException {
        writer.setCurrentFile(inFile.substring(1+inFile.lastIndexOf('\\'), inFile.lastIndexOf('.')).toUpperCase());

        Parser parser = new Parser(inFile);

        //goes through parsed .vm file writing assembly to a new .asm file
        while(parser.hasMoreLines()){
            parser.advance();
            //You can't change it to a switch statement and make the enumeration globally available
            if(parser.commandType() == commandType.C_PUSH || parser.commandType() == commandType.C_POP) {

                writer.writePushPop(parser.commandType(), parser.arg1(), parser.arg2());
            } else if(parser.commandType() == commandType.C_ARITHMETIC){

                writer.writeArithmetic(parser.arg1());
            } else if(parser.commandType() == commandType.C_LABEL){

                writer.writeLabel(parser.arg1());
            } else if(parser.commandType() == commandType.C_GOTO){

                writer.writeGoto(parser.arg1());
            }else if(parser.commandType() == commandType.C_IF){

                writer.writeIf(parser.arg1());
            }else if(parser.commandType() == commandType.C_CALL){

                writer.writeCall(parser.arg1(), parser.arg2());
            } else if(parser.commandType() == commandType.C_FUNCTION){

                writer.writeFunction(parser.arg1(), parser.arg2());
            } else if(parser.commandType() == commandType.C_RETURN) {
                writer.writeReturn();
            }
        }
    }
}
