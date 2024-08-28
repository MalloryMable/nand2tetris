import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.regex.Pattern;
import java.util.regex.Matcher;
import java.util.stream.Collectors;

public class JackTokenizer {
    public enum tokenType {KEYWORD, SYMBOL, IDENTIFIER, INT_CONST, STRING_CONST}
    public static final List<String> keywords = List.of("class", "constructor", "function", "method", "field",
            "static", "var", "int", "char", "boolean", "void", "true", "false", "null", "this", "let", "do", "if",
            "else", "while", "return");
    private static final Pattern mainPat = Pattern.compile(
            "\"[^\"]*\"|\\s?;|[{}\\[\\],.+\\-*&|<>/=~()]\\s?|\\d+\\s?|\\w+\\s?");
    private static final Pattern commentPat = Pattern.compile("//");
    private static final Pattern digitPat = Pattern.compile("\\d+");
    private static final Pattern stringPat = Pattern.compile("\"([^\"]*)");
    private static final Pattern symbolPat = Pattern.compile("\\s?([;{}\\[\\],.+\\-*&|<>/=~()])\\s?");

    private final Matcher matcher;

    private String stringConst;
    private char symbol;
    private int intVal;
    private String identifier;
    private int keywordIndex;
    private tokenType type;

    public JackTokenizer(Path inFile) throws IOException {

        String fileString = Files.lines(inFile)
                .map(string -> {
                    Matcher commentMatch = commentPat.matcher(string); // removes double dash comments01
                    return commentMatch.find() ? string.substring(0, commentMatch.start()) : string;
                }) // trims comments
                .map(string -> string.replaceAll("\\t+", "") // removes all tabs
                        .replaceAll("\\s+", " ") // trims sets of spaces to one space
                        .replaceAll("^\\s|\\s$", "")) // removes leading and trailing space
                .collect(Collectors.joining())// joins stream of lines into a single string
                .replaceAll("/\\*.+?\\*/", ""); // finally remove block comments from the string

        matcher = mainPat.matcher(fileString);
    }

    public boolean hasMoreTokens() {
        return matcher.find();
    }

    public void advance() {
        // I do not reset metadata and trust that only the correct functions will be called for different types of data
        String tokenString = matcher.group();

        Matcher stringMatcher = stringPat.matcher(tokenString);
        Matcher sybmolMatcher = symbolPat.matcher(tokenString);
        Matcher digitMatcher = digitPat.matcher(tokenString);

        if(sybmolMatcher.find()){
            type = tokenType.SYMBOL;
            symbol = sybmolMatcher.group(1).charAt(0);
        } else if(digitMatcher.find()) {
            type = tokenType.INT_CONST;
            intVal = Integer.parseInt(digitMatcher.group()); // this is always going to be digits so no catching needed
        } else if(stringMatcher.find()){
            type = tokenType.STRING_CONST;
            stringConst = stringMatcher.group(1); // for some reason this was taking group zero ?
        } else {
            tokenString = tokenString.strip(); // easier to strip here than fool around with groups
            if(keywords.contains(tokenString)){
                type = tokenType.KEYWORD;
                keywordIndex = keywords.indexOf(tokenString);
            } else {
                type = tokenType.IDENTIFIER;
                identifier = tokenString;
            }
        }
    }
    public tokenType getType(){
        return type;
    }
    public String identifier(){
        return identifier;
    }
    public String stringVal(){
        return stringConst;
    }
    public char symbol(){
        return symbol;
    }
    public int intVal(){
        return intVal;
    }
    public String keyword(){
        return keywords.get(keywordIndex);
    }
}
