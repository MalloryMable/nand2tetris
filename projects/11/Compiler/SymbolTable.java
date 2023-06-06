import java.util.ArrayList;

public class SymbolTable {

    int staticCount, fieldCount, argCount, varCount;
    public enum kind {STATIC, FIELD, ARG, VAR}
    private final ArrayList<Symbol> symbolTable;

    public SymbolTable() {
        symbolTable = new ArrayList<>();
        staticCount = 0;
        fieldCount = 0;
        argCount = 0;
        varCount = 0;
    }
    public void define(String type, String name , kind defineKind) {

        //ordering change here reflects the change from read in to storage order
        symbolTable.add(new Symbol(name, type, defineKind, switch (defineKind){
            case STATIC -> staticCount++;
            case FIELD -> fieldCount++;
            case ARG -> argCount++;
            case VAR -> varCount++;
        }));
    }

    public void startSubroutine(){
         symbolTable.removeIf(Symbol::localScope);
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

    public kind kindOf(String name){  return symbolIndex(name).getKind();
    }

    public int indexOf(String name){
        return symbolIndex(name).getIndex();
    }

    //TODO: Make this use a hash table
    private Symbol symbolIndex(String name) {
        for(Symbol symbol: symbolTable){
            if(symbol.getName().equals(name)){
                return symbol;
            }
        }
        return null;
    }

    private static class Symbol{
        String name;
        String type;
        kind kind;
        int index;
        private Symbol(String name, String type,kind kind, int index){
            this.name = name;
            this.type = type;
            this.kind = kind;
            this.index = index;
        }

        private boolean localScope(){
            return kind == SymbolTable.kind.VAR || kind == SymbolTable.kind.ARG;
        }

        private String getName(){return name;}
        private String getType(){return type;}
        private kind getKind() { return kind;}

        public int getIndex() {return index;}
    }
}
