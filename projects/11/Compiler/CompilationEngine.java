import java.io.IOException;
import java.nio.file.Path;

public class CompilationEngine {

    /*This program makes heavy use of enums, these are preferable to bulky
    * Strings getting passed around or harder to parse arrays of numbers
    * with an insubstantial change in efficiency between the two*/
    public enum segment {CONST, ARG, LOCAL, STATIC, THIS, THAT, POINTER, TEMP}
    public enum operation {MULT, DIV, ADD, SUB, NEG, EQ, GT, LT, AND, OR, NOT}
    private final VMWriter writer;
    private final SymbolTable symbolTable;
    private final JackTokenizer tokenizer;
    private String className;
    private int labelCount;

    public CompilationEngine(Path path) throws IOException {
        tokenizer = new JackTokenizer(path);
        writer = new VMWriter(Path.of( path.toString().replace(".jack", ".vm")));
        symbolTable = new SymbolTable(); //assigned in constructor and not compile class so table may be final
        labelCount = 0;
        compileClass();
    }


    //wrapper function called on initialization
    public void compileClass(){
        //begins reading
        safeAdvance();


        safeAdvance();
        className = tokenizer.identifier(); //Class name
        safeAdvance();

        checkSymbol('{');

        compileClassVarDec();
        compileSubroutine();
        writer.close();
    }

    //prints the initial variables declared at the top of the file(with class declaration)
    public void compileClassVarDec(){
        //allows for rows of declared variables

        while (tokenizer.keyword().equals("static") || tokenizer.keyword().equals("field")) {

            //SCOPE Declaration stores one of these
            SymbolTable.kind scope = tokenizer.keyword().equals("static") ?
                    SymbolTable.kind.STATIC: SymbolTable.kind.FIELD;
            safeAdvance();
            String type = getDecType();

            symbolTable.define(type, tokenizer.identifier(), scope);
            safeAdvance();

            // checks for multiple declared variables
            while (tokenizer.symbol() == ',') {
                //advances past comma
                safeAdvance();

                symbolTable.define(type, tokenizer.identifier(), scope);
                safeAdvance();
            }
            checkSymbol(';');
        }
    }

    //prints a type declaration, some number of variables, and any number of statements until '}' is reached
    public void compileSubroutine(){
        String functionName;
        // breaks the recursive call
        if (isSymbol() && tokenizer.symbol() == '}') {
            return;
        }

        //throws an error if a subroutine is declared incorrectly
        if (!(tokenizer.keyword().equals("function")
                || tokenizer.keyword().equals("method")
                || tokenizer.keyword().equals("constructor"))) {
            throw new RuntimeException("Expected subroutine declaration");

        }
        //Begins subroutine
        symbolTable.startSubroutine();
        //the choice here is to advance inside the if statement or make the keyword a variable to advance past it
        if(tokenizer.keyword().equals("constructor")){
            //finds the number of field variables and sets a given object to need that much space to construct itself
            writer.writePush(segment.CONST, symbolTable.varCount(SymbolTable.kind.FIELD));
            writer.writeCall("Memory.alloc", 1);
            writer.writePop(segment.POINTER, 0);

            safeAdvance();
            //type collection omitted
            safeAdvance();
            functionName = tokenizer.identifier();
            safeAdvance();
            checkSymbol('(');
            writer.writeFunction(
                    String.format("%s.%s", className, functionName),
                    compileParameterList());
        } else if(tokenizer.keyword().equals("function")){
            safeAdvance();
            //omitted return type getter as this is a weakly typed language
            safeAdvance();
            functionName = tokenizer.identifier();
            safeAdvance();
            checkSymbol('(');

            writer.writeFunction(
                    String.format("%s.%s", className, functionName),
                    compileParameterList());

        } else {
            safeAdvance();
            //advance past object type
            safeAdvance();
            functionName = tokenizer.identifier();
            safeAdvance();
            checkSymbol('(');
            //one added to the number of args needed as this is always pushed as the first argument in a method
            writer.writeFunction(
                    String.format("%s.%s", className, functionName),
                    1 + compileParameterList());
            //pushes this to symbol table, pushes it to argument zero pops to pointer so this becomes our local object
            symbolTable.define(className, "this", SymbolTable.kind.ARG);
            writer.writePush(segment.ARG, 0);
            writer.writePop(segment.POINTER, 0);

        }
        //closed parenthesis is already checked so the program simply advance past it
        safeAdvance();

        checkSymbol('{');
        // get all var declarations in the subroutine
        while (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD && tokenizer.keyword().equals("var") ) {
            safeAdvance();
            compileVarDec();
        }

        compileStatements();
        //no need to check for close curly bracket
        // recursive call
        compileSubroutine();
    }

    //takes any number of data types and variable names divided by commas
    public int compileParameterList(){
        int count = 0;
        while(!(isSymbol() && tokenizer.symbol() == ')')){
            //gets name and also non keyword class types
            symbolTable.define( getDecType(), tokenizer.identifier(), SymbolTable.kind.ARG);
            safeAdvance();
            // for comma separated lists
            if (isSymbol() && tokenizer.symbol() ==  ',') {
                safeAdvance();
            }
        }
        return count;
    }

    // prints statements until closed curly bracket is reached
    public void compileStatements(){
        while(!(isSymbol() && tokenizer.symbol() == '}')){
            if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD) {
                switch (tokenizer.keyword()) {

                    case "do" -> compileDo();
                    case "let" -> compileLet();
                    case "if" -> compileIf();
                    case "while" -> compileWhile();
                    case "return" -> compileReturn();
                    default -> throw new RuntimeException("not a function keyword: " + tokenizer.keyword());
                }
            }

        }
        //advance past closed curly bracket that ended the statement set
        safeAdvance();

    }

    //prints any number of declared variables at the top of a subroutine
    public void compileVarDec(){
        String type = getDecType();
        //first name declaration

        symbolTable.define(type, tokenizer.identifier(), SymbolTable.kind.VAR);
        safeAdvance();

        //optional further declarations
        while (JackTokenizer.tokenType.SYMBOL == tokenizer.getType() && tokenizer.symbol() == ','){
            safeAdvance();
            symbolTable.define(type, tokenizer.identifier(), SymbolTable.kind.VAR);
            safeAdvance();
        }
        checkSymbol(';');
    }

    //prints do statement
    public void compileDo(){
        String functionName;
        safeAdvance();

        // function call
        functionName = tokenizer.identifier();
        safeAdvance();
        // for dot statement calls(ex: foo.bar())
        if (isSymbol() && tokenizer.symbol() == '.') {
            safeAdvance();
            functionName = String.format("%s.%s",functionName, tokenizer.identifier());
            safeAdvance();
        }else {
            functionName = String.format("%s.%s",className, functionName);
            writer.writePush(segment.THIS, 0);
        }

        checkSymbol('(');
        // parameters in the parentheses
        writer.writeCall(functionName,compileExpressionList());
        checkSymbol(')');
        checkSymbol(';');
    }

    //prints a let statement
    public void compileLet(){
        safeAdvance();

        String variable = tokenizer.identifier();
        safeAdvance();
        // in the case of an array foo[x]
        if (isSymbol() && tokenizer.symbol() == '[') {

            writer.writePush(translateSegment(symbolTable.kindOf(variable)), symbolTable.indexOf(variable));
            safeAdvance();

            compileExpression();
            writer.writeArithmetic(operation.ADD);
            writer.writePop(segment.POINTER, 1);
            checkSymbol(']');
            checkSymbol('=');
            //pushes the to be popped object
            compileExpression();
            writer.writePop(segment.THAT, 0);
        } else {

            checkSymbol('=');
            compileExpression();
            writer.writePop(translateSegment(symbolTable.kindOf(variable)), symbolTable.indexOf(variable));
        }

    checkSymbol(';');
    }

    //compiles a while statement
    public void compileWhile(){
        String breakLabel = String.format("L%d", labelCount++);
        String continueLabel = String.format("L%d", labelCount++);

        writer.writeLabel(continueLabel);
        safeAdvance();
        checkSymbol('(');
        compileExpression();
        writer.writeArithmetic(operation.EQ);//ensures that the while is not false(ie that pushed value is 0)
        writer.writeGoto(breakLabel);
        checkSymbol(')');

        checkSymbol('{');

        compileStatements();
        writer.writeGoto(continueLabel);
        writer.writeLabel(breakLabel);

    }

    //prints return statement called from Statement
    public void compileReturn(){
        safeAdvance();
        if ((isSymbol() && tokenizer.symbol() == ';')) {
            writer.writePush(segment.CONST, 0);
            writer.writeReturn();

        } else {
            compileExpression();
        }
        checkSymbol(';');
    }

    //prints an if statement
    public void compileIf(){
        String falseLabel = String.format("L%d", labelCount++);
        String trueLabel = String.format("L%d", labelCount++);

        safeAdvance();

        checkSymbol('(');

        compileExpression();
        writer.writeArithmetic(operation.EQ);//checks if pushed function eq 0
        writer.writeGoto(falseLabel);

        checkSymbol(')');

        checkSymbol('{');
        compileStatements();
        // closed curly bracket is implicit
        // if there is an else clause of the if statement
        if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD && tokenizer.keyword().equals("else")) {
            writer.writeGoto(trueLabel);
            writer.writeLabel(falseLabel);
            safeAdvance();

            checkSymbol('{');

            compileStatements();

            writer.writeLabel(trueLabel);
        } else {
            writer.writeLabel(falseLabel);
        }
    }

    // pushes one to two terms combined by an optional binary operator
    public void compileExpression(){
        compileTerm();
        if (isSymbol() && isOperation()) {
            operation opp = switch (tokenizer.symbol()) {
                case '+' -> operation.ADD;
                case '<' -> operation.LT;
                case '>' -> operation.GT;
                case '&' -> operation.AND;
                case '=' -> operation.EQ;
                case '-' -> operation.SUB;
                case '*' -> operation.MULT;
                case '/' -> operation.DIV;
                default -> throw new RuntimeException("Expected operator symbol instead of" + tokenizer.symbol());
            };
            safeAdvance();
            compileTerm();
            switch (opp){
                case MULT -> writer.writeCall("Math.multiply()", 2);
                case DIV -> writer.writeCall("Math.divide()", 2);
                default -> writer.writeArithmetic(opp);

            }
        }
    }

    //pushes a possible variable including arrays (foo[5]) and dot functions foo.bar() as well as unary operators
    public void compileTerm(){

        //First case, identifier call
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {

            String functionName = tokenizer.identifier();
            safeAdvance();

            //array reference foo[3]
            if (isSymbol() && tokenizer.symbol() == '[') {
                writer.writePush(translateSegment(symbolTable.kindOf(functionName)), symbolTable.indexOf(functionName));
                safeAdvance();

                compileExpression();
                writer.writeArithmetic(operation.ADD);
                writer.writePop(segment.POINTER, 1); //sets pointer one then looks at what is stored there
                writer.writePush(segment.THAT,0);



            }
            //function call. Either Foo.bar() or foo()
            else if (isSymbol() &&
                    (tokenizer.symbol() == '(' || tokenizer.symbol() == '.')) {
                //either way this is now a function call
                // for dot statement calls(ex: foo.bar())
                if (tokenizer.symbol() == '.') {
                    safeAdvance();
                    functionName = String.format("%s.%s",functionName, tokenizer.identifier());
                    safeAdvance();

                }else {
                    functionName = String.format("%s.%s", className, functionName);
                }

                checkSymbol('(');
                //writes call to invoked function with count taken from expression list
                writer.writeCall(functionName, compileExpressionList());
                checkSymbol(')');
                //for vanilla variables(ex: foo)
            } else {
                writer.writePush(translateSegment(symbolTable.kindOf(functionName)), symbolTable.indexOf(functionName));
            }
        //Built in data types
        } else {
            // integer
            if (tokenizer.getType() == JackTokenizer.tokenType.INT_CONST) {
                writer.writePush(segment.CONST, tokenizer.intVal());
                safeAdvance();
            }
            // strings
            else if (tokenizer.getType() == JackTokenizer.tokenType.STRING_CONST) {
                for(int charCount = 0; charCount < tokenizer.stringVal().length(); charCount++){
                    //pushes the length of the string to a new string
                    writer.writePush(segment.CONST, tokenizer.stringVal().length());
                    writer.writeCall("String.new", 1);
                    writer.writePush(segment.CONST, tokenizer.stringVal().charAt(charCount));
                    writer.writeCall("String.append", 1);
                }
                safeAdvance();
            }
            //TODO:may need to implement char

            // this, true, false, or null
            else if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD &&
                    (tokenizer.keyword().equals("this") || tokenizer.keyword().equals("null") ||
                            tokenizer.keyword().equals("false") || tokenizer.keyword().equals("true"))) {
                switch (tokenizer.keyword()){
                    case "this" -> writer.writePush(segment.THIS, 0);
                    case "null", "false" -> writer.writePush(segment.CONST, 0);
                    case "true" ->{
                        writer.writePush(segment.CONST, 1);
                        writer.writeArithmetic(operation.NEG);
                    }
                }
                safeAdvance();

            }
            // parenthetical separation
            else if (isSymbol() && tokenizer.symbol() == '(') {
                safeAdvance();

                compileExpression();

                checkSymbol(')');
            }
            // unary operators
            else if (isSymbol() &&
                    (tokenizer.symbol() == '-' || tokenizer.symbol() == '~')) {

                operation opp = tokenizer.symbol() == '-'? operation.NEG: operation.NOT;
                safeAdvance();

                compileTerm();
                writer.writeArithmetic(opp);
            }
        }
    }

    //recursive function that takes any number of expressions divided by commas
    public int compileExpressionList(){
        int count = 0;
        if(isSymbol() && tokenizer.symbol() == ')'){
            return count;
        }
        compileExpression();
        count++;
        //do while not used because of the need to advance past the comma
        while (isSymbol() && tokenizer.symbol()== ','){
            safeAdvance();
            compileExpression();
            count++;
        }
        return count;
    }

    //An advance function that throws an error if the program ends unexpectedly
    private void safeAdvance(){
        if(tokenizer.hasMoreTokens()) {
            tokenizer.advance();
        } else {
            throw new RuntimeException("Runtime Error. Unexpected program end");
        }
    }

    //checks for a given symbol and returns an error if language syntax isn't met
    private void checkSymbol(char symbol){
        if(!(isSymbol() && tokenizer.symbol() == symbol)){
            throw new RuntimeException("Runtime Error. Expected token:" + symbol);
        }
        safeAdvance();
    }

    //checks if the current data type is built in or constructed, then advances
    private String getDecType(){
        String type = tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER ?
                tokenizer.identifier(): tokenizer.keyword();
        safeAdvance();
        return type;
    }

    //checks that current tokens type is a symbol
    private boolean isSymbol() {
        return tokenizer.getType() == JackTokenizer.tokenType.SYMBOL;
    }

    //broken out logic from compile expression checking for one of the four possible chars
    private boolean isOperation(){
        return "<>+-=&*/".contains(String.valueOf(tokenizer.symbol()));
    }

    private segment translateSegment(SymbolTable.kind kind){
        return switch (kind){
            case STATIC -> segment.STATIC;
            case FIELD -> segment.THIS;
            case ARG -> segment.ARG;
            case VAR -> segment.LOCAL;
        };
    }
    //TODO: and a keyword check like symbol check to make reading and catching bad code easier
}
