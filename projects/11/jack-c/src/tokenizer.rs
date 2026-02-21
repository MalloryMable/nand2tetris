use std::iter::Peekable;
use std::str::Chars;

// Define toekns
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: TokenType,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(Keyword),
    Symbol(Symbol),
    Identifier(String),
    IntConst(i16),
    StringConst(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Op(Operator),
    Delim(Delimiter),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus, Minus, Star, Slash, Ampersand, Pipe, LessThan, GreaterThan, Equal, Tilde,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    OpenBrace, CloseBrace, OpenParen, CloseParen, OpenBracket, CloseBracket, Dot, Comma, Semicolon,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Class, Constructor, Function, Method, Field, Static, Var, Int, Char, Boolean,
    Void, True, False, Null, This, Let, Do, If, Else, While, Return,
}

impl Symbol {
    pub fn from_char(c: char) -> Option<Symbol> {
        match c {
            '{' => Some(Symbol::Delim(Delimiter::OpenBrace)),
            '}' => Some(Symbol::Delim(Delimiter::CloseBrace)),
            '(' => Some(Symbol::Delim(Delimiter::OpenParen)),
            ')' => Some(Symbol::Delim(Delimiter::CloseParen)),
            '[' => Some(Symbol::Delim(Delimiter::OpenBracket)),
            ']' => Some(Symbol::Delim(Delimiter::CloseBracket)),
            '.' => Some(Symbol::Delim(Delimiter::Dot)),
            ',' => Some(Symbol::Delim(Delimiter::Comma)),
            ';' => Some(Symbol::Delim(Delimiter::Semicolon)),
            '+' => Some(Symbol::Op(Operator::Plus)),
            '-' => Some(Symbol::Op(Operator::Minus)),
            '*' => Some(Symbol::Op(Operator::Star)),
            // NOTE: Slash is handled in skip_trivia due to comment ambiguity
            '\\' => Some(Symbol::Op(Operator::Slash)),
            '&' => Some(Symbol::Op(Operator::Ampersand)),
            '|' => Some(Symbol::Op(Operator::Pipe)),
            '<' => Some(Symbol::Op(Operator::LessThan)),
            '>' => Some(Symbol::Op(Operator::GreaterThan)),
            '=' => Some(Symbol::Op(Operator::Equal)),
            '~' => Some(Symbol::Op(Operator::Tilde)),
            _ => None,
        }
    }
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s {
            // NOTE: These are case sensistive so we don't correct these
            "class" => Some(Keyword::Class),
            "constructor" => Some(Keyword::Constructor),
            "function" => Some(Keyword::Function),
            "method" => Some(Keyword::Method),
            "field" => Some(Keyword::Field),
            "static" => Some(Keyword::Static),
            "var" => Some(Keyword::Var),
            "int" => Some(Keyword::Int),
            "char" => Some(Keyword::Char),
            "boolean" => Some(Keyword::Boolean),
            "void" => Some(Keyword::Void),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            "this" => Some(Keyword::This),
            "let" => Some(Keyword::Let),
            "do" => Some(Keyword::Do),
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "return" => Some(Keyword::Return),
            _ => None,
        }
    }
}


pub struct JackTokenizer<'a> {
    source: Peekable<Chars<'a>>,
    line_number: usize,
}

impl<'a> JackTokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            line_number: 1,
        }
    }

    // -- helper: comments --

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.source.next() {
            if c == '\n' {
                self.line_number += 1;
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        let mut closed = false;
        while let Some(c) = self.source.next() {
            if c == '\n' { self.line_number += 1; }

            if c == '*' {
                if let Some(&'/') = self.source.peek() {
                    self.source.next(); // Consume closing '/'
                    closed = true;
                    break;
                }
            }
        }

        if closed {
            Ok(())
        } else {
            Err(format!("Unclosed block comment starting at line {}", self.line_number))
        }
    }

    // --- Helper: Token Parsers ---

    fn read_integer(&mut self, first_digit: char) -> Result<Token, String> {
            let line = self.line_number;
            let mut val = first_digit.to_string();

            while let Some(&c) = self.source.peek() {
                if c.is_digit(10) {
                    val.push(self.source.next().unwrap());
                } else {
                    break;
                }
            }

            match val.parse::<i16>() {
                Ok(n) => Ok(Token { value: TokenType::IntConst(n), line }),
                Err(_) => Err(format!("Int constant too large (must be <= 32767) on line {}", line)),
            }
        }
    fn read_string(&mut self) -> Result<Token, String> {
        let line = self.line_number;
        let mut val = String::new();

        loop {
            match self.source.next() {
                Some('"') => break,
                Some('\n') => return Err(format!("Newline in string literal line {}", line)),
                Some(c) => val.push(c),
                None => return Err(format!("Unterminated string literal line {}", line)),
            }
        }

        Ok(Token { value: TokenType::StringConst(val), line })
    }

    fn read_identifier_or_keyword(&mut self, first_char: char) -> Token {
        let line = self.line_number;
        let mut val = first_char.to_string();

        while let Some(&c) = self.source.peek() {
            if c.is_alphanumeric() || c == '_' {
                val.push(self.source.next().unwrap());
            } else {
                break;
            }
        }

        let token_type = match Keyword::from_str(&val) {
            Some(k) => TokenType::Keyword(k),
            None => TokenType::Identifier(val),
        };

        Token { value: token_type, line }
    }
}


impl<'a> Iterator for JackTokenizer<'a> {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = *self.source.peek()?; // Peeks and checks for EOF

            if c.is_whitespace() {
                if c == '\n' { self.line_number += 1; }
                self.source.next(); // Consume and loop again
                continue;
            }

            // comments and blank spaces
            if c == '/' {
                self.source.next(); // Consume the '/'

                match self.source.peek() {
                    Some('/') => {
                        self.source.next(); // Consume the second '/'
                        self.skip_line_comment();
                        continue; // Loop again (trivia)
                    }
                    Some('*') => {
                        self.source.next(); // Consume the '*'
                        if let Err(e) = self.skip_block_comment() {
                            return Some(Err(e));
                        }
                        continue; // Loop again (trivia)
                    }
                    _ => {
                        // NOTE: we treat '/' as a valid divisor
                        return Some(Ok(Token {
                            value: TokenType::Symbol(Symbol::Op(Operator::Slash)),
                            line: self.line_number,
                        }));
                    }
                }
            }

            // This could still be EOF
            let c = self.source.next()?;

            // Now we do the meat and potatos tokenization
            if c.is_digit(10) {
                return Some(self.read_integer(c));
            } else if c == '"' {
                return Some(self.read_string());
            } else if c.is_alphabetic() || c == '_' {
                return Some(Ok(self.read_identifier_or_keyword(c)));
            } else {
                return match Symbol::from_char(c) {
                    Some(sym) => Some(Ok(Token {
                        value: TokenType::Symbol(sym),
                        line: self.line_number,
                    })),
                    None => Some(Err(format!("Unexpected char '{}' on line {}", c, self.line_number))),
                };
            }
        }
    }
}
