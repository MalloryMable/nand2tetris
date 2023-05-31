import java.io.*;
import java.nio.file.Path;


public class CompilationEngine {
    private final PrintWriter writer;
    private final JackTokenizer tokenizer;
    private String indent = "\t";
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
        writer.format("%s<keyword> class </keyword>\n", indent);

        safeAdvance();
        writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());

        safeAdvance();
        writer.format("%s<symbol> %c </symbol>\n",indent, tokenizer.symbol());

        safeAdvance();
        compileClassVarDec();

        compileSubroutine();

        writer.format("%s<symbol> %c </symbol>\n",indent, tokenizer.symbol());
        writer.println("</class>\n");

        writer.close();
    }

    //prints the initial variables declared at the top of the file(with class declaration)
    public void compileClassVarDec(){
        //allows for rows of declared variables
        while (tokenizer.keyword().equals("static") || tokenizer.keyword().equals("field")) {
            writer.format("%s<classVarDec>\n", indent);
                // field or static
                addIndent();
                writer.format("%s<keyword> %s </keyword>\n",indent , tokenizer.keyword());
                safeAdvance();

                printDatatype();
                // the name section of variable declaration field int [num]
                writer.format("%s<identifier> %s </identifier>\n",indent , tokenizer.identifier());
                safeAdvance();
                // checks for multiple declared variables
                while (tokenizer.symbol() == ',') {
                    writer.format("%s<symbol> , </symbol>\n", indent);
                    safeAdvance();

                    writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
                    safeAdvance();
                }
                // this should be semicolon every time
                writer.format("%s<symbol> ; </symbol>\n", indent);

                decIndent();
                writer.format("%s</classVarDec>\n", indent);
                safeAdvance();
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

            writer.format("%s<subroutineDec>\n", indent);
            addIndent();
            writer.format("%s<keyword> %s </keyword>\n",indent, tokenizer.keyword());
        }

        safeAdvance();

        // if the subroutine uses a class datatype OR we are encountering a constructor
        printDatatype();

        // name of the subroutine if not constructor
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
            safeAdvance();
        }
        //returns an error language syntax is broken
        checkSymbol('(');

        writer.format("%s<parameterList>\n", indent);
        addIndent();

        compileParameterList();

        decIndent();
        writer.format("%s</parameterList>\n", indent);
        writer.format("%s<symbol> ) </symbol>\n",indent);
        //advance outside of if statement
        safeAdvance();

        writer.format("%s<subroutineBody>\n", indent);
        addIndent();
        checkSymbol('{');

        // get all var declarations in the subroutine
        while (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD && tokenizer.keyword().equals("var") ) {
            writer.format("%s<varDec>\n", indent);
            addIndent();
            writer.format("%s<keyword> var </keyword>\n", indent);
            safeAdvance();
            compileVarDec();
            decIndent();
            writer.format("%s</varDec>\n", indent);
        }

        writer.format("%s<statements>\n", indent);
        addIndent();
        compileStatements();
        decIndent();
        writer.format("%s</statements>\n", indent);
        writer.format("%s<symbol> %c </symbol>\n", indent, tokenizer.symbol());

        decIndent();
        writer.format("%s</subroutineBody>\n", indent);
        decIndent();
        writer.format("%s</subroutineDec>\n", indent);


        // recursive call
        compileSubroutine();

        }

    //takes any number of data types and variable names divided by commas
    public void compileParameterList(){
        while(!(isSymbol() && tokenizer.symbol() == ')')){
            //gets name and also non keyword class types
            if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
                writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
            } else {
                writer.format("%s<keyword> %s </keyword>\n",indent, tokenizer.keyword());
            }
            safeAdvance();
            // for comma separated lists
            if (isSymbol() && tokenizer.symbol() ==  ',') {
                writer.format("%s<symbol> , </symbol>\n", indent);
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
                    writer.format("%s<doStatement>\n", indent);
                    addIndent();
                    compileDo();
                    decIndent();
                    writer.format("%s</doStatement>\n", indent);
                }
                case "let" -> {
                    writer.format("%s<letStatement>\n", indent);
                    addIndent();
                    compileLet();
                    decIndent();
                    writer.format("%s</letStatement>\n", indent);
                }
                case "if" -> {
                    writer.format("%s<ifStatement>\n", indent);
                    addIndent();
                    compileIf();
                    decIndent();
                    writer.format("%s</ifStatement>\n", indent);
                }
                case "while" -> {
                    writer.format("%s<whileStatement>\n", indent);
                    addIndent();
                    compileWhile();
                    decIndent();
                    writer.format("%s</whileStatement>\n", indent);
                }
                 case "return"-> {
                     writer.format("%s<returnStatement>\n", indent);
                     addIndent();compileReturn();
                     decIndent();
                     writer.format("%s</returnStatement>\n", indent);
                }
            }
        }
        safeAdvance();
        compileStatements();
    }

    //prints any number of declared variables at the top of a subroutine
    public void compileVarDec(){

        //type declaration
        printDatatype();

        //first name declaration
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
        }
        safeAdvance();

        //optional further declarations
        while (JackTokenizer.tokenType.SYMBOL == tokenizer.getType() && tokenizer.symbol() == ','){
            writer.format("%s<symbol> , </symbol>\n", indent);
            safeAdvance();
            writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
            safeAdvance();
        }

        if (JackTokenizer.tokenType.SYMBOL == tokenizer.getType() && tokenizer.symbol() == ';') {
            writer.format("%s<symbol> ; </symbol>\n", indent);
        }
        safeAdvance();
    }

    //prints do statement
    public void compileDo(){
        writer.format("%s<keyword> do </keyword>\n", indent);
        safeAdvance();

        // function call
        writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
        safeAdvance();
        // for dot statement calls(ex: foo.bar())
        if (isSymbol() && tokenizer.symbol() == '.') {
            writer.format("%s<symbol> . </symbol>\n", indent);
            safeAdvance();

            writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
            safeAdvance();
        }

        checkSymbol('(');

        // parameters in the parentheses
        writer.format("%s<expressionList>\n", indent);
        addIndent();
        compileExpressionList();
        decIndent();
        writer.format("%s</expressionList>\n", indent);

        checkSymbol(')');
    }

    //prints a let statement
    public void compileLet(){
        writer.format("%s<keyword> let </keyword>\n", indent);
        safeAdvance();

        writer.format("%s<identifier> %s </identifier>\n",indent , tokenizer.identifier());
        safeAdvance();
        // in the case of an array foo[x]
        if (isSymbol() && tokenizer.symbol() == '[') {
            writer.format("%s<symbol> [ </symbol>\n", indent);
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
        writer.format("%s<keyword> while </keyword>\n", indent);
        safeAdvance();

        checkSymbol('(');
        compileExpression();
        checkSymbol('(');

        checkSymbol('{');

        writer.format("%s<statements>\n", indent);
        addIndent();
        compileStatements();
        decIndent();
        writer.format("%s</statements>\n", indent);

        checkSymbol('}');
    }

    //prints return statement called from Statement
    public void compileReturn(){
        writer.format("%s<keyword> return </keyword>\n", indent);
        safeAdvance();
        if (!(isSymbol() && tokenizer.symbol() == ';')) {
            compileExpression();
        }
        checkSymbol(';');
    }

    //prints an if statement
    public void compileIf(){
        writer.format("%s<keyword> if </keyword>\n", indent);
        safeAdvance();

        checkSymbol('(');

        compileExpression();

        checkSymbol(')');

        checkSymbol('{');

        writer.format("%s<statements>\n", indent);
        addIndent();
        compileStatements();
        decIndent();
        writer.format("%s</statements>\n", indent);

        checkSymbol('}');

        // if there is an else clause of the if statement
        if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD && tokenizer.keyword().equals("else")) {
            writer.format("%s<keyword> else </keyword>\n",indent);
            safeAdvance();

            checkSymbol('{');

            writer.format("%s<statements>\n", indent);
            addIndent();
            compileStatements();
            decIndent();
            writer.format("%s</statements>\n", indent);

            checkSymbol('}');
        }
    }

    // prints one to two terms(see below) combined by an optional binary operator
    public void compileExpression(){
        writer.format("%s<expression>\n", indent);
        addIndent();
        compileTerm();

        while (isSymbol() && isOperation()) {
            // < > & = have different xml code
            switch (tokenizer.symbol()) {
                case '<' -> writer.format("%s<symbol> &lt; </symbol>\n", indent);
                case '>' -> writer.format("%s<symbol> &gt; </symbol>\n", indent);
                case '&' -> writer.format("%s<symbol> &amp; </symbol>\n", indent);
                case '=' -> writer.format("%s<symbol> = </symbol>\n", indent);
                default -> throw new RuntimeException("Expected operator symbol instead of" + tokenizer.symbol());
            }
            safeAdvance();

            compileTerm();
        }
        decIndent();
        writer.format("%s</expression>\n", indent);
    }

    //prints a possible variable including arrays (foo[5]) and dot functions foo.bar() as well as unary operators
    public void compileTerm(){
        writer.format("%s<term>\n", indent);
        addIndent();
        //First case, identifier call
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            String prevIdentifier = tokenizer.identifier();
            safeAdvance();
            // for [] terms
            if (isSymbol() && tokenizer.symbol() == '[') {
                writer.format("%s<identifier> %s </identifier>\n",indent , prevIdentifier);
                writer.format("%s<symbol> [ </symbol>\n", indent);
                safeAdvance();
                compileExpression();

                safeAdvance();
                writer.format("%s<symbol> ] </symbol>\n", indent);
            }
            //  subroutine calls both dot '.' and parenthetic '()' calls
            else if (isSymbol() &&
                    (tokenizer.symbol() == '(' || tokenizer.symbol() == '.')) {

                // for dot statement calls(ex: foo.bar())
                if (tokenizer.symbol() == '.') {
                    writer.format("%s<identifier> %s </identifier>\n",indent, prevIdentifier);
                    writer.format("%s<symbol> . </symbol>\n", indent);
                    safeAdvance();

                    writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
                    safeAdvance();
                }

                checkSymbol('(');

                writer.format("%s<expressionList>\n", indent);
                addIndent();
                compileExpressionList();
                decIndent();
                writer.format("%s</expressionList>\n", indent);

                checkSymbol(')');
            //for vanilla variables(ex: foo)
            } else {
                writer.format("%s<identifier> %s </identifier>\n",indent, prevIdentifier);
            }
        //Built in data types
        } else {
            // integer
            if (tokenizer.getType() == JackTokenizer.tokenType.INT_CONST) {
                writer.format("%s<integerConstant> %d </integerConstant>\n",indent , tokenizer.intVal());
                safeAdvance();
            }
            // strings
            else if (tokenizer.getType() == JackTokenizer.tokenType.STRING_CONST) {
                writer.format("%s<stringConstant> %s </stringConstant>\n",indent , tokenizer.stingVal());
                safeAdvance();
            }
            // this, true, false, or null
            else if (tokenizer.getType() == JackTokenizer.tokenType.KEYWORD &&
                    (tokenizer.keyword().equals("this") || tokenizer.keyword().equals("null") ||
                    tokenizer.keyword().equals("false") || tokenizer.keyword().equals("true"))) {
                writer.format("%s<keyword> %s </keyword>\n",indent, tokenizer.keyword());
                safeAdvance();

            }
            // parenthetical separation
            else if (isSymbol() || tokenizer.symbol() == '(') {
                writer.format("%s<symbol> ( </symbol>\n", indent);
                safeAdvance();

                compileExpression();

                checkSymbol(')');
            }
            // unary operators
            else if (isSymbol() &&
                    (tokenizer.symbol() == '-' || tokenizer.symbol() == '~')) {

                writer.format("<symbol> %s </symbol>\n", tokenizer.symbol());
                safeAdvance();

                compileTerm();
                safeAdvance();
            }
        }
        decIndent();
        writer.format("%s</term>\n",indent);

    }

    //recursive function that takes any number of expressions divided by commas
    public void compileExpressionList(){
        if(isSymbol() && tokenizer.symbol() == ')'){
            return;
        }
        compileExpression();

        while (isSymbol() && tokenizer.symbol()== ','){

            writer.format("%s<symbol> , </symbol>\n", indent);
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

    //checks for a given symbol and returns an error if language syntax isn't met
    private void checkSymbol(char symbol){
        if(!(isSymbol() && tokenizer.symbol() == symbol)){
            throw new RuntimeException("Runtime Error. Expected token:" + symbol);
        }
        writer.format("%s<symbol> %c </symbol>\n",indent, tokenizer.symbol());
        safeAdvance();
    }

    //checks if the current data type is built in or constructed, then advances
    private void printDatatype(){
        if (tokenizer.getType() == JackTokenizer.tokenType.IDENTIFIER) {
            writer.format("%s<identifier> %s </identifier>\n",indent, tokenizer.identifier());
        } else {
            writer.format("%s<keyword> %s </keyword>\n",indent, tokenizer.keyword());
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

    //adds 1 tab to indentation
    private void addIndent(){
        indent += '\t';
    }

    //removes the first tab from indentation
    private void decIndent(){
        indent = indent.substring(1);
    }
}
