import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;


public class HackAssembler {
    public static void main(String[] args) throws IOException {
        //Takes an argument or the present file.asm value
        String inFile = args.length != 0 ? args[0] : "file.asm";
        String outFile = inFile.substring(0,inFile.lastIndexOf('.')) + ".hack";

        Parser parser = new Parser(inFile);
        SymbolTable symbolTable = new SymbolTable();

        int adjustedCount = 0;  //tracks the line count after symbols are removed!
        //FIRST PASS: adds labels to symbol table
        while (parser.hasMoreLines()) {
            parser.advance();

            if (parser.instructionType() == Parser.instructionType.L_COMMAND) {
                symbolTable.addEntry(parser.symbol(), adjustedCount);
            } else {
                adjustedCount++;
            }
        }
        //adjusted count altered to now serve as an address counter
        adjustedCount = 16;

        //SECOND PASS: identify instruction type, translate instructions into binary, write binary to a new file
        PrintWriter writer = new PrintWriter(new FileWriter(outFile));
        while (parser.hasMoreLines()) {
            parser.advance();

            if (parser.instructionType() == Parser.instructionType.C_COMMAND) {
                //parsed into binary in the order of Comp, dest, jump
                writer.printf("111%s%s%s\n", Code.comp(parser.comp()), Code.dest(parser.dest()), Code.jump(parser.jump()));

            } else if (parser.instructionType() == Parser.instructionType.A_COMMAND) {
                String symbol = parser.symbol();

                //if parsing to int fails there is a String symbol which might be in the symbolList
                try {
                    writer.println(symbolToBin(Integer.parseInt(symbol)));
                } catch (NumberFormatException e) {
                    //checks if the String is in the symbolTable. If not the new string is added
                    if (symbolTable.contains(symbol)) {
                        writer.println(symbolToBin(symbolTable.getAddress(symbol)));
                    } else {
                        writer.println(symbolToBin(adjustedCount));
                        symbolTable.addEntry(symbol, adjustedCount++);
                    }
                }
            }
        }
        writer.close();
    }

    //turns a given number ito a binary representation and backfills with zero up to the 16th place
    private static String symbolToBin(int symbolValue) {
        return String.format("%16s", Integer.toBinaryString(symbolValue)).replace(' ', '0');
    }
}

