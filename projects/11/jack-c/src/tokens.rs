use std::fmt;

use crate::symbol_table::Primitive;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: TokenType,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keywd(Keyword),
    Symbol(Symbol),
    Id(String),
    IntConst(usize),
    StrgConst(String),
}

// --- Token Types ---
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Prim(Primitive),
    Scope(Scope),
    Action(Action),
    Routine(Routine),
    Const(Const),
    Class,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Static, Field, Var, // Keyword-derived
    Arg, Const, This, That, Pointer, Temp // VM-derived
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Op(Operator),
    Delim(Delimiter),
}

// --- Keywords ---
#[derive(Debug, Clone, PartialEq)]
pub enum Action { Let, Do, If, Else, While, Return }

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive { Int, Char, Bool, Void }

#[derive(Debug, Clone, PartialEq)]
pub enum Routine { Constructor, Function, Method }

#[derive(Debug, Clone, PartialEq)]
pub enum Const { True, False, Null, This }


// --- Symbols ---
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add, Sub, Mult, Divide, And, Or, Lesser, Greater, Equal, Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    OpenBrace, CloseBrace, OpenParen, CloseParen, OpenBracket, CloseBracket,
    Dot, Comma, Semicolon,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s {
            "int" => Some(Keyword::Prim(Primitive::Int)),
            "char" => Some(Keyword::Prim(Primitive::Char)),
            "boolean" => Some(Keyword::Prim(Primitive::Bool)),
            "void" => Some(Keyword::Prim(Primitive::Void)),
            "static" => Some(Keyword::Scope(Scope::Static)),
            "field" => Some(Keyword::Scope(Scope::Field)),
            "var" => Some(Keyword::Scope(Scope::Var)),
            "let" => Some(Keyword::Action(Action::Let)),
            "do" => Some(Keyword::Action(Action::Do)),
            "if" => Some(Keyword::Action(Action::If)),
            "else" => Some(Keyword::Action(Action::Else)),
            "while" => Some(Keyword::Action(Action::While)),
            "return" => Some(Keyword::Action(Action::Return)),
            "constructor" => Some(Keyword::Routine(Routine::Constructor)),
            "function" => Some(Keyword::Routine(Routine::Function)),
            "method" => Some(Keyword::Routine(Routine::Method)),
            "true" => Some(Keyword::Const(Const::True)),
            "false" => Some(Keyword::Const(Const::False)),
            "null" => Some(Keyword::Const(Const::Null)),
            "this" => Some(Keyword::Const(Const::This)),
            "class" => Some(Keyword::Class),
            _ => None,
        }
    }
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
            '+' => Some(Symbol::Op(Operator::Add)),
            '-' => Some(Symbol::Op(Operator::Sub)),
            '*' => Some(Symbol::Op(Operator::Mult)),
            '/' => Some(Symbol::Op(Operator::Divide)),
            '&' => Some(Symbol::Op(Operator::And)),
            '|' => Some(Symbol::Op(Operator::Or)),
            '<' => Some(Symbol::Op(Operator::Lesser)),
            '>' => Some(Symbol::Op(Operator::Greater)),
            '=' => Some(Symbol::Op(Operator::Equal)),
            '~' => Some(Symbol::Op(Operator::Not)),
            _ => None,
        }
    }
}


// --- Print Methods ---
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Let => write!(f, "let"),
            Action::Do => write!(f, "do"),
            Action::If => write!(f, "if"),
            Action::Else => write!(f, "else"),
            Action::While => write!(f, "while"),
            Action::Return => write!(f, "return"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Keywd(k) => write!(f, "{}", k),
            TokenType::Symbol(s) => write!(f, "{}", s),
            TokenType::Id(s) => write!(f, "{}", s),
            TokenType::IntConst(i) => write!(f, "{}", i),
            TokenType::StrgConst(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keyword::Prim(k) => write!(f, "{}", k),
            Keyword::Scope(k) => write!(f, "{}", k),
            Keyword::Action(k) => write!(f, "{}", k),
            Keyword::Routine(k) => write!(f, "{}", k),
            Keyword::Const(k) => write!(f, "{}", k),
            Keyword::Class => write!(f, "class"),
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Static => write!(f, "static"),
            Scope::Field => write!(f, "field"),
            Scope::Var => write!(f, "var"),
            Scope::Arg => write!(f, "argument"),
            Scope::Const => write!(f, "constant"),
            Scope::This => write!(f, "this"),
            Scope::That => write!(f, "that"),
            Scope::Pointer => write!(f, "pointer"),
            Scope::Temp => write!(f, "temp"),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Primitive::Int => write!(f, "int"),
            Primitive::Char => write!(f, "char"),
            Primitive::Bool => write!(f, "boolean"),
            Primitive::Void => write!(f, "void"),
        }
    }
}


impl fmt::Display for Routine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Routine::Constructor => write!(f, "constructor"),
            Routine::Function => write!(f, "function"),
            Routine::Method => write!(f, "method"),
        }
    }
}


impl fmt::Display for Const {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Const::True => write!(f, "true"),
            Const::False => write!(f, "false"),
            Const::Null => write!(f, "null"),
            Const::This => write!(f, "this"),
        }
    }
}
impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Op(op) => write!(f, "{}", op),
            Symbol::Delim(d) => write!(f, "{}", d),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mult => "*",
            Operator::Divide => "/",
            Operator::And => "&",
            Operator::Or => "|",
            Operator::Lesser => "<",
            Operator::Greater => ">",
            Operator::Equal => "=",
            Operator::Not => "~",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Delimiter::OpenBrace => "{",
            Delimiter::CloseBrace => "}",
            Delimiter::OpenParen => "(",
            Delimiter::CloseParen => ")",
            Delimiter::OpenBracket => "[",
            Delimiter::CloseBracket => "]",
            Delimiter::Dot => ".",
            Delimiter::Comma => ",",
            Delimiter::Semicolon => ";",
        };
        write!(f, "{}", s)
    }
}
