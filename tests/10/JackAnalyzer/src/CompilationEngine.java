import java.io.*;
import java.nio.file.Path;


public class CompilationEngine {
    private final PrintWriter writer;
    private final JackTokenizer tokenizer;
    private String indent = "\t";
    private static final String identifier = "identifier";
    private static final String keyword = "keyword";
    public CompilationEngine(Path path) throws IOException {
        tokenizer = new JackTokenizer(path);

        Path outFile = Path.of( path.toString().replace(".jack", ".xml"));
        writer = new PrintWriter(outFile.toFile());

        compileClass();
    }


    //wrapper function called on initialization
    public void compileClass(){
        safeAdvance();
        writer.println("<class>");

        printEnclosed( keyword, "class");
        safeAdvance();

        printEnclosed(identifier, tokenizer.identifier());
        safeAdvance();

        checkSymbol('{');

        compileClassVarDec();

        compileSubroutine();

        //this is always checked by compileSubroutine
        printSymbol(tokenizer.symbol());
        writer.println("</class>\n");

        writer.close();
    }

    //prints the initial variables declared at the top of the file(with class declaration)
    public void compileClassVarDec(){
        //allows for rows of declared variables
        while (tokenizer.keyword().equals("static") || tokenizer.keyword().equals("field")) {
            printOpen("classVarDec");
                // field or static

                printEnclosed(keyword, tokenizer.keyword());
                safeAdvance();

                compileDatatype();
                // the name section of variable declaration field int [num]
                printEnclosed(identifier, tokenizer.identifier());

                safeAdvance();
                // checks for multiple declared variables
                while (tokenizer.symbol() == ',') {
                    printSymbol(',');
                    safeAdvance();

                    printEnclosed(identifier, tokenizer.identifier());
                    safeAdvance();
                }
                // this should be semicolon every time
                checkSymbol(';');

                printClose("classVarDec");
            }
    }

    //prints a type declaration, some number of variables, and any number of statements until '}' is reached
    public void compileSubroutine(){
        // breaks the recursive call

        if (isSymbol() && tokenizer.symbol() == '}') {
            safeAdvance();
            return;
        }

        //Begins subroutine
        if (tokenizer.keyword().equals("function") || tokenizer.keyword().equals("method") || tokenizer.keyword().equals("constructor")) {

            printOpen("subroutineDec");

            printEnclosed(keyword, tokenizer.keyword());
        }

        safeAdvance();

        // if the subroutine uses a class datatype OR we are encountering a constructor
        compileDatatype();

        // name of the subroutine if not constructor
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            printEnclosed(identifier, tokenizer.identifier());
            safeAdvance();
        }
        //returns an error language syntax is broken
        checkSymbol('(');

        printOpen("parameterList");

        compileParameterList();

        printClose("parameterList");
         printSymbol(')');
        //advance outside of if statement
        safeAdvance();

        printOpen("subroutineBody");
        checkSymbol('{');

        // get all var declarations in the subroutine
        while (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD && tokenizer.keyword().equals("var") ) {
            printOpen("varDec");

            printEnclosed(keyword, "var");
            safeAdvance();
            compileVarDec();

            printClose("varDec");
        }

        printOpen("statements");

        compileStatements();

        printClose("statements");
        printSymbol(tokenizer.symbol());

        printClose("subroutineBody");
        printClose("subroutineDec");


        // recursive call
        compileSubroutine();
        }

    //takes any number of data types and variable names divided by commas
    public void compileParameterList(){
        while(!(isSymbol() && tokenizer.symbol() == ')')){
            //gets name and also non keyword class types
            compileDatatype();
            printEnclosed(identifier, tokenizer.identifier());
            safeAdvance();
            // for comma separated lists
            if (isSymbol() && tokenizer.symbol() ==  ',') {
                printSymbol(',');
                safeAdvance();
            }

        }
    }

    //recursively prints any number of statements until a '}' is reached
    public void compileStatements(){
        if (isSymbol() && tokenizer.symbol() == '}') {
            return;
        } else if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD ) {
            switch (tokenizer.keyword()) {
                case "do" -> {
                    printOpen("doStatement");
                    compileDo();
                    printClose("doStatement");
                }
                case "let" -> {
                    printOpen("letStatement");
                    compileLet();
                    printClose("letStatement");
                }
                case "if" -> {
                    printOpen("ifStatement");
                    compileIf();
                    printClose("ifStatement");
                }
                case "while" -> {
                    printOpen("whileStatement");
                    compileWhile();
                    printClose("whileStatement");
                }
                 case "return"-> {
                     printOpen("returnStatement");
                     compileReturn();
                     printClose("returnStatement");
                }
            }
        }
        safeAdvance();
        compileStatements();
    }

    //prints any number of declared variables at the top of a subroutine
    public void compileVarDec(){

        //type declaration
        compileDatatype();

        //first name declaration
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            printEnclosed(identifier, tokenizer.identifier());
        }
        safeAdvance();

        //optional further declarations
        while (JackTokenizer.tokenType.SYMBOL == tokenizer.getType() && tokenizer.symbol() == ','){
            printSymbol(',');
            safeAdvance();
            printEnclosed(identifier, tokenizer.identifier());
            safeAdvance();
        }

        checkSymbol(';');
    }

    //prints do statement
    public void compileDo(){
        printEnclosed(keyword, "do");
        safeAdvance();

        // function call
        printEnclosed(identifier, tokenizer.identifier());
        safeAdvance();
        // for dot statement calls(ex: foo.bar())
        if (isSymbol() && tokenizer.symbol() == '.') {
            printSymbol('.');
            safeAdvance();

            printEnclosed(identifier, tokenizer.identifier());
            safeAdvance();
        }

        checkSymbol('(');

        // parameters in the parentheses
        printOpen("expressionList");
        compileExpressionList();
        printClose("expressionList");

        checkSymbol(')');
    }

    //prints a let statement
    public void compileLet(){
        printEnclosed(keyword, "let");
        safeAdvance();

        printEnclosed(identifier, tokenizer.identifier());
        safeAdvance();
        // in the case of an array foo[x]
        if (isSymbol() && tokenizer.symbol() == '[') {
            printSymbol('[');

            compileExpression();
            safeAdvance();

            checkSymbol(']');
        }

        checkSymbol('=');

        compileExpression();

        checkSymbol(';');
    }

    //compiles a while statement
    public void compileWhile(){
        printEnclosed(keyword, "while" );
        safeAdvance();

        checkSymbol('(');
        compileExpression();
        checkSymbol('(');

        checkSymbol('{');

        printOpen("statements");
        compileStatements();
        printClose("statements");

        checkSymbol('}');
    }

    //prints return statement called from Statement
    public void compileReturn(){
        printEnclosed(keyword,"return");
        safeAdvance();
        if (!(isSymbol() && tokenizer.symbol() == ';')) {
            compileExpression();
        }
        checkSymbol(';');
    }

    //prints an if statement
    public void compileIf(){
        printEnclosed(keyword, "if");
        safeAdvance();

        checkSymbol('(');

        compileExpression();

        checkSymbol(')');

        checkSymbol('{');

        printOpen("statements");
        compileStatements();
        printClose("statements");

        checkSymbol('}');

        // if there is an else clause of the if statement
        if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD && tokenizer.keyword().equals("else")) {
            printEnclosed(keyword, "else");
            safeAdvance();

            checkSymbol('{');

            printOpen("statements");
            compileStatements();
            printClose("statements");

            checkSymbol('}');
        }
    }

    // prints one to two terms(see below) combined by an optional binary operator
    public void compileExpression(){
        printOpen("expression");

        compileTerm();

        while (isSymbol() && isOperation()) {
            // < > & = have different xml code
            switch (tokenizer.symbol()) {
                case '<' -> printEnclosed("symbol", "&lt;");
                case '>' -> printEnclosed("symbol","&gt;");
                case '&' -> printEnclosed("symbol","&amp;");
                case '=' -> printSymbol('=');
                default -> throw new RuntimeException("Expected operator symbol instead of" + tokenizer.symbol());
            }
            safeAdvance();

            compileTerm();
        }
        printClose("expression");
    }

    //prints a possible variable including arrays (foo[5]) and dot functions foo.bar() as well as unary operators
    public void compileTerm(){
        printOpen("term");

        //First case, identifier call
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            String prevIdentifier = tokenizer.identifier();
            safeAdvance();
            // for [] terms
            if (isSymbol() && tokenizer.symbol() == '[') {
                printEnclosed(identifier, prevIdentifier);
                printSymbol('[');
                safeAdvance();
                compileExpression();

                safeAdvance();
                //this should be checked but advancing breaks things
                printSymbol(']');
            }
            //  subroutine calls both dot '.' and parenthetic '()' calls
            else if (isSymbol() &&
                    (tokenizer.symbol() == '(' || tokenizer.symbol() == '.')) {

                // for dot statement calls(ex: foo.bar())
                if (tokenizer.symbol() == '.') {
                    printEnclosed(identifier, prevIdentifier);
                    printSymbol('.');
                    safeAdvance();

                    printEnclosed(identifier, tokenizer.identifier());
                    safeAdvance();
                }

                checkSymbol('(');

                printOpen("expressionList");
                compileExpressionList();
                printClose("%expressionList");

                checkSymbol(')');
            //for vanilla variables(ex: foo)
            } else {
                printEnclosed(identifier, prevIdentifier);
            }
        //Built in data types
        } else {
            // integer
            if (tokenizer.getType() == JackTokenizer.tokenType.INT_CONST) {
                print(String.format("<integerConstant> %d </integerConstant>" , tokenizer.intVal()));
                safeAdvance();
            }
            // strings
            else if (tokenizer.getType() == JackTokenizer.tokenType.STRING_CONST) {
                printEnclosed("stringConstant" , tokenizer.stingVal());
                safeAdvance();
            }
            // this, true, false, or null
            else if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD &&
                    (tokenizer.keyword().equals("this") || tokenizer.keyword().equals("null") ||
                    tokenizer.keyword().equals("false") || tokenizer.keyword().equals("true"))) {
                printEnclosed(keyword, tokenizer.keyword());
                safeAdvance();

            }
            // parenthetical separation
            else if (isSymbol() || tokenizer.symbol() == '(') {
                printSymbol('(');
                safeAdvance();

                compileExpression();

                checkSymbol(')');
            }
            // unary operators
            else if (isSymbol() &&
                    (tokenizer.symbol() == '-' || tokenizer.symbol() == '~')) {

                printSymbol(tokenizer.symbol());
                safeAdvance();

                compileTerm();
                safeAdvance();
            }
        }

        printClose("term");

    }

    //recursive function that takes any number of expressions divided by commas
    public void compileExpressionList(){
        if(isSymbol() && tokenizer.symbol() == ')'){
            return;
        }
        compileExpression();

        while (isSymbol() && tokenizer.symbol()== ','){

            printSymbol( ',');
            safeAdvance();
            compileExpression();
        }
}

    //An advance function that throws an error if the program ends unexpectedly
    private void safeAdvance(){
        if(tokenizer.hasMoreTokens()) {
            tokenizer.advance();
        } else {
            throw new RuntimeException("Runtime Error. Unexpected program end");
        }
    }

    private void print(String text){
        writer.format("%s%s\n", indent, text);
    }
    private void printSymbol(char symbol){
        print(String.format("<symbol> %c </symbol>", symbol));
    }

    private void printEnclosed(String tag, String text){
        print(String.format("<%1$s> %2$s </%1$s>", tag, text));
    }

    private void printOpen(String tag){
        print(String.format("<%s>", tag));
        indent+= '\t';
    }

    private void printClose(String tag){
        indent = indent.substring(1);
        print(String.format("</%s>", tag));
    }

    //checks for a given symbol and returns an error if language syntax isn't met
    private void checkSymbol(char symbol){
        if(!(isSymbol() && tokenizer.symbol() == symbol)){
            throw new RuntimeException("Runtime Error. Expected token:" + symbol);
        }
        printSymbol(tokenizer.symbol());
        safeAdvance();
    }

    //checks if the current data type is built in or constructed, then advances
    private void compileDatatype(){
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            printEnclosed(identifier, tokenizer.identifier());
        } else {
            printEnclosed(keyword, tokenizer.keyword());
        }
        safeAdvance();
    }

    //checks that current tokens type is a symbol
    private boolean isSymbol() {
        return tokenizer.getType() == JackTokenizer.tokenType.SYMBOL;
    }

    //broken out logic from compile expression checking for one of the four possible chars
    private boolean isOperation(){
        return "<>+=".contains(String.valueOf(tokenizer.symbol()));
    }

}
