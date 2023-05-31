import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.stream.Stream;

public class JackAnalyzer {
    public static void main(String[] args) throws IOException {
        Path path = Path.of(args[0]);

        //checks if the user input is a directory or single file and uses the appropriate logic
        if((Files.isDirectory(path))){
            //found out streams do this recursion for you and also learned lambda notation
            try (Stream<Path> stream = Files.walk(Path.of(path.toUri()))) {
                stream.filter(Files::isRegularFile)
                        .filter(n -> n.getFileName().toString().contains(".jack"))
                        .forEach(inFile -> {
                            try {
                                new CompilationEngine(inFile);
                            } catch (IOException e) {
                                throw new RuntimeException(e);
                            }
                        });//This only isn't a double colon because of I/O exceptions
            }
        } else {
            //rather than creating the filename string twice to trim the file type a variable is created
            new CompilationEngine(path);
        }
    }
    //This is broken out to more easily interact with its logic more easily
    private static void analyze(Path path) throws IOException {
        JackTokenizer tokenizer = new JackTokenizer(path);
        while (tokenizer.hasMoreTokens()){
            tokenizer.advance();
            switch (tokenizer.getType()){
                case KEYWORD -> {
                    switch(tokenizer.keyword()){

                    }
                }
                case SYMBOL -> tokenizer.symbol();
                case IDENTIFIER -> tokenizer.identifier();
                case INT_CONST -> tokenizer.intVal();
                case STRING_CONST -> tokenizer.stingVal();
                default -> System.out.println("common failure state");
            }
        }
    }
}