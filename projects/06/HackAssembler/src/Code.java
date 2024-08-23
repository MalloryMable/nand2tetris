public class Code {

    //returns 7 digits of computation code
    public static String comp(String line){
        return switch (line) {
            case "0" -> "0101010";
            case "1" -> "0111111";
            case "A" -> "0110000";
            case "M" -> "1110000";
            case "D" -> "0001100";
            case "-1" -> "0111010";
            case "!D" -> "0001101";
            case "!A" -> "0110001";
            case "!M" -> "1110001";
            case "-D" -> "0001111";
            case "-A" -> "0110011";
            case "-M" -> "1110011";
            case "D-1" -> "0001110";
            case "A-1" -> "0110010";
            case "M-1" -> "1110010";
            case "D-A" -> "0010011";
            case "D+1", "1+D" -> "0011111";
            case "A+1", "1+A" -> "0110111";
            case "M+1", "1+M" -> "1110111";
            case "D-M" -> "1010011";
            case "A-D" -> "0000111";
            case "M-D" -> "1000111";
            case "D+A", "A+D" -> "0000010";
            case "D+M", "M+D" -> "1000010";
            case "D&A", "A&D" -> "0000000";
            case "D&M", "M&D" -> "1000000";
            case "D|A", "A|D" -> "0010101";
            case "D|M", "M|D" -> "1010101";
            default -> throw new IllegalArgumentException(line + ": Is an invalid comp instruction");
        };
    }

    //returns a 3 digit destination code
    public static String dest(String line){
        String destBin = "000";

        if(line == null) {
            return destBin;
        }

        //Sets a destination flag to true or false using String Builder
        StringBuilder sb = new StringBuilder(destBin);
        if(line.contains("A")){
            sb.setCharAt(0, '1');
        }
        if(line.contains("D")){
            sb.setCharAt(1, '1');
        }
        if(line.contains("M")){
            sb.setCharAt(2, '1');
        }
        return sb.toString();
    }


    //returns a 3 digit jump code
    public static String jump(String line){
        if(line == null) {
            return "000";
        }
        return switch (line) {
            case "JGT" -> "001";
            case "JEQ" -> "010";
            case "JGE" -> "011";
            case "JLT" -> "100";
            case "JNE" -> "101";
            case "JLE" -> "110";
            case "JMP" -> "111";
            default -> throw new IllegalArgumentException(line + ": Is an invalid jump instruction");
        };
    }
}
