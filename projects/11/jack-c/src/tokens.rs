use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: TokenType,
    pub line: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keywd(Keyword),
    Symbol(Symbol),
    Id(String),
    IntConst(i16),
    StrgConst(String),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Type(Type),
    Scope(Scope),
    Action(Action),
    Routine(Routine),
    Const(Const),
    Class,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s {
            "int" => Some(Keyword::Type(Type::Int)),
            "char" => Some(Keyword::Type(Type::Char)),
            "boolean" => Some(Keyword::Type(Type::Bool)),
            "void" => Some(Keyword::Type(Type::Void)),
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

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keyword::Type(k) => write!(f, "{}", k),
            Keyword::Scope(k) => write!(f, "{}", k),
            Keyword::Action(k) => write!(f, "{}", k),
            Keyword::Routine(k) => write!(f, "{}", k),
            Keyword::Const(k) => write!(f, "{}", k),
            Keyword::Class => write!(f, "class"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type { Int, Char, Bool, Void }
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Char => write!(f, "char"),
            Type::Bool => write!(f, "boolean"),
            Type::Void => write!(f, "void"),
        }
    }
}

// Merged Enum: Contains both keyword scopes (Static/Field/Var)
// and VM-only segments (Arg/Const/etc)
#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Static, Field, Var, // Keyword-derived
    Arg, Const, This, That, Pointer, Temp // VM-derived
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

#[derive(Debug, Clone, PartialEq)]
pub enum Action { Let, Do, If, Else, While, Return }
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

#[derive(Debug, Clone, PartialEq)]
pub enum Routine { Constructor, Function, Method }
impl fmt::Display for Routine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Routine::Constructor => write!(f, "constructor"),
            Routine::Function => write!(f, "function"),
            Routine::Method => write!(f, "method"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Const { True, False, Null, This }
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

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Op(Operator),
    Delim(Delimiter),
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

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Op(op) => write!(f, "{}", op),
            Symbol::Delim(d) => write!(f, "{}", d),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add, Sub, Mult, Divide, And, Or, Lesser, Greater, Equal, Not,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    OpenBrace, CloseBrace, OpenParen, CloseParen, OpenBracket, CloseBracket, Dot, Comma, Semicolon,
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
