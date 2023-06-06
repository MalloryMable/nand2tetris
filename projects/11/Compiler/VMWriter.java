import java.io.FileNotFoundException;
import java.io.PrintWriter;
import java.nio.file.Path;

public class VMWriter {
    //Might make Segments and arithmetic hard coded arrays and have all their references be single numbers
    PrintWriter writer;
    public VMWriter(Path outFile) throws FileNotFoundException {
        writer = new PrintWriter(outFile.toFile());
    }

    public void writePush(CompilationEngine.segment segment, int index){
        writer.format("\tpush %s %d\n" ,segmenttoString(segment), index );
    }

    public void writePop(CompilationEngine.segment segment, int index){
        writer.format("\tpush %s %d\n" ,segmenttoString(segment), index );
    }

    public void writeArithmetic(CompilationEngine.operation operation){
        writer.format("\t%s\n", switch (operation){
            case ADD -> "add";
            case SUB -> "sub";
            case NEG -> "neg";
            case EQ -> "eq";
            case GT -> "gt";
            case LT -> "lt";
            case AND -> "and";
            case OR -> "or";
            case NOT -> "not";
            default -> throw new RuntimeException("Tried to run an invalid form of arithmetic");
        });
    }

    public void writeLabel(String label){
        writer.format("label %s\n", label);
    }

    public void writeGoto(String label){
        writer.format("\tgoto %s\n", label);
    }

    public void writeCall(String name, int nArgs){
        writer.format("\tcall %s %d\n", name, nArgs);
    }

    public void writeFunction(String name, int nLocals){
        writer.format("function %s %d\n", name, nLocals);
    }

    public void writeReturn(){
        //push local 0 ?
        writer.println("\treturn");
    }

    public void close(){
        writer.close();
    }

    private String segmenttoString(CompilationEngine.segment segment){
        return switch (segment) {
            case CONST -> "constant";
            case ARG -> "argument";
            case LOCAL -> "local";
            case STATIC -> "static";
            case THIS -> "this";
            case THAT -> "that";
            case POINTER -> "pointer";
            case TEMP -> "temporary";
        };
    }

}
