import java.util.ArrayList;

public class SymbolTable {
    private final ArrayList<String> symbolList= new ArrayList<>();
    private final ArrayList<Integer> addressList = new ArrayList<>();

    public SymbolTable(){
        //initialize table here
        addEntry("R0", 0);
        addEntry("R1", 1);
        addEntry("R2", 2);
        addEntry("R3", 3);
        addEntry("R4", 4);
        addEntry("R5", 5);
        addEntry("R6", 6);
        addEntry("R7", 7);
        addEntry("R8", 8);
        addEntry("R9", 9);
        addEntry("R10", 10);
        addEntry("R11", 11);
        addEntry("R12", 12);
        addEntry("R13", 13);
        addEntry("R14", 14);
        addEntry("R15", 15);
        addEntry("SCREEN", 16384);
        addEntry("KBD", 24576);
        addEntry("SP", 0);
        addEntry("LCL", 1);
        addEntry("ARG", 2);
        addEntry("THIS", 3);
        addEntry("THAT", 4);
    }

    //Add a symbol and its given address
    public void addEntry(String symbol, int address){
        symbolList.add(symbol);
        addressList.add(address);
    }

    //checks for a given symbol in the list
    public boolean contains(String symbol){
        return symbolList.contains(symbol);
    }

    //Returns the address of a given symbol. Use only if symbolList contains symbol
    public int getAddress(String symbol){
        return addressList.get(symbolList.indexOf(symbol));
    }
}
