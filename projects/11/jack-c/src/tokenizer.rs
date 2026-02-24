use std::iter::Peekable;
use std::str::Chars;

use crate::tokens::{self, Token, TokenType, Symbol, Operator, Delimiter, Keyword, Segment};

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

    //  Helper: comments and whitespace

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

    //  Helper: token parsers

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

        match val.parse::<usize>() {
            //TODO: Also check explicitly if the parsed int is <= 32767 because of how ints are held
            Ok(n) => Ok(Token { value: TokenType::IntConst(n), line }),
            Err(_) => Err(format!("{}: Int constant too large (must be <= 32767)", line)),
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let line = self.line_number;
        let mut val = String::new();

        loop {
            match self.source.next() {
                Some('"') => break,
                Some('\n') => return Err(format!("{}: Newline in string literal", line)),
                Some(c) => val.push(c),
                None => return Err(format!("{}: Unterminated string literal", line)),
            }
        }

        Ok(Token { value: TokenType::StrgConst(val), line })
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

        // Try keyword
        if let Some(k) = Keyword::from_str(&val) {
            return Token { value: TokenType::Keywd(k), line };
        }

        // Try segment (static, field, var)
        if let Some(s) = Segment::from_str(&val) {
            return Token { value: TokenType::Segment(s), line };
        }

        // Fallback to identifier
        Token { value: TokenType::Id(val), line }
    }
}

impl<'a> Iterator for JackTokenizer<'a> {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = *self.source.peek()?;

            // Whitespace
            if c.is_whitespace() {
                if c == '\n' { self.line_number += 1; }
                self.source.next();
                continue;
            }

            // Comments
            if c == '/' {
                self.source.next();

                match self.source.peek() {
                    Some('/') => {
                        self.source.next();
                        self.skip_line_comment();
                        continue;
                    }
                    Some('*') => {
                        self.source.next();
                        if let Err(e) = self.skip_block_comment() {
                            return Some(Err(e));
                        }
                        continue;
                    }
                    _ => {
                        // Just a slash operator
                        return Some(Ok(Token {
                            value: TokenType::Symbol(Symbol::Op(Operator::Divide)),
                            line: self.line_number,
                        }));
                    }
                }
            }

            let c = self.source.next()?;

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
                    None => Some(Err(format!("{}: Unexpected char '{}'", c, self.line_number))),
                };
            }
        }
    }
}
