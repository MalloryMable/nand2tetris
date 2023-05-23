import java.io.IOException;

public class Main {

    enum commandType {
        C_ARITHMATIC, C_PUSH, C_POP
    }

    public static void main(String[] args) throws IOException {
        String inFile = args.length != 0 ? args[0] : "file.vm";

        Parser parser = new Parser(inFile);
        CodeWriter writer = new CodeWriter(inFile.substring(0,inFile.lastIndexOf('.')));

        //goes through parsed .vm file writing assembly to a new .asm file
        while(parser.hasMoreLines()){
            parser.advance();
            if(parser.commandType() == commandType.C_PUSH || parser.commandType() == commandType.C_POP) {
                writer.writePushPop(parser.commandType(), parser.arg1(), parser.arg2());
            } else{
                writer.writeArithmetic(parser.arg1());
            }
        }
        writer.close();

    }
}
