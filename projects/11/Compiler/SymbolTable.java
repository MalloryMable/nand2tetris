import java.util.HashMap;

public class SymbolTable {

    int staticCount, fieldCount, argCount, varCount;
    public enum kind {STATIC, FIELD, ARG, VAR}
    private final HashMap<String, Symbol> symbolTable;

    public SymbolTable() {
        symbolTable = new HashMap<>();
        staticCount = 0;
        fieldCount = 0;
        argCount = 0;
        varCount = 0;
    }
    public void define(String type, String name , kind defineKind) {

        //ordering change here reflects the change from read in to storage order
        symbolTable.put(name, new Symbol( type, defineKind, switch (defineKind){
            case STATIC -> staticCount++;
            case FIELD -> fieldCount++;
            case ARG -> argCount++;
            case VAR -> varCount++;
        }));
    }

    public void startSubroutine(){
         symbolTable.entrySet().removeIf(entry -> entry.getValue().localScope());
         argCount = 0;
         varCount = 0;
    }

    public int varCount(kind kindOf){
        return switch (kindOf){
            case STATIC -> staticCount;
            case FIELD -> fieldCount;
            case ARG -> argCount;
            case VAR -> varCount;
        };
    }

    public kind kindOf(String name){  return symbolTable.get(name).getKind();
    }

    public int indexOf(String name){
        return symbolTable.get(name).getIndex();
    }

    private static class Symbol{
        //String name;
        String type;
         kind kind;
        int index;
        private Symbol(String type,kind kind, int index){
            this.type = type;
            this.kind = kind;
            this.index = index;
        }

        private boolean localScope(){
            return kind == SymbolTable.kind.VAR || kind == SymbolTable.kind.ARG;
        }

        //commented out but left in to keep the option of Typing open in the future
        //private String getType(){return type;}
        private kind getKind() { return kind;}

        public int getIndex() {return index;}

    }
}
