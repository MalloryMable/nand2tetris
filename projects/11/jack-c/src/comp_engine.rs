use std::iter::Peekable;
use crate::tokenizer::JackTokenizer;
use crate::tokens::{
    Token, TokenType, Keyword, Symbol, Delimiter, Operator,
    Segment, Action, Routine, Const, Type
};

#[derive(Debug, Clone)]
pub enum TypeDescriptor {
    Primitive(Type),
    Class(String),
}

pub struct CompilationEngine<'a, W: VMWriter> {
    tokenizer: Peekable<JackTokenizer<'a>>,
    writer: &'a mut W,
    symbol_table: SymbolTable,
    class_name: String,
    label_count: usize,
}

impl<'a, W: VMWriter> CompilationEngine<'a, W> {
    pub fn new(tokenizer: JackTokenizer<'a>, writer: &'a mut W) -> Self {
        Self {
            tokenizer: tokenizer.peekable(),
            writer,
            // TODO: SymbolTable should be passed from main
            symbol_table: SymbolTable::new(),
            class_name: String::new(),
            label_count: 0,
        }
    }

    // --- Compilation Logic ---

    pub fn compile_class(&mut self) -> Result<(), String> {
        let token = self.advance()?;
        match token.value {
            TokenType::Keywd(Keyword::Class) => {},
            _ => return Err(format!("{}: Expected 'class' declaration, found '{}'", token.line, token)),
        }
        self.class_name = self.expect_ident()?;
        self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;

        // Optional Class variable declarations
        while let Ok(token) = self.peek() {
            if let TokenType::Segment(seg) = &token.value {
                match seg {
                    Segment::Static | Segment::Field => self.compile_class_var_dec()?,
                    _ => break,
                }
            } else {
                break;
            }
        }

        // Subroutine Declarations
        while let Ok(token) = self.peek() {
            if let TokenType::Keywd(Keyword::Routine(_)) = &token.value {
                self.compile_subroutine()?;
            } else {
                break;
            }
        }

        self.expect_symbol(Symbol::Delim(Delimiter::CloseBrace))?;
        // TODO: Flush
        Ok(())
    }

    fn compile_class_var_dec(&mut self) -> Result<(), String> {
        // Peeked and consumed as variable declaration is optional
        let token = self.advance()?;

        let segment = match token.value {
            TokenType::Segment(s) => s,
            _ => return Err(format!("{}: Expected static or field, found '{}'", token.line, token)),
        };

        let type_desc = self.get_type_descriptor()?;

        // Vectorized: Consume "x, y, z;" and define all of them
        let names = self.expect_ident_list()?;
        for name in names {
            self.symbol_table.define(&name, &type_desc, segment.clone());
        }

        Ok(())
    }

    fn compile_subroutine(&mut self) -> Result<(), String> {
        self.symbol_table.start_subroutine();

        let token = self.advance()?;
        let routine_kind = match token.value {
            TokenType::Keywd(Keyword::Routine(r)) => r,
            _ => return Err(format!("{}: Expected constructor, function, or method, found '{}'", token.line, token)),
        };

        let _routine_return_type = self.get_type_descriptor()?;

        let func_name = self.expect_ident()?;
        let full_name = format!("{}.{}", self.class_name, func_name);

        self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;

        if let Routine::Method = routine_kind {
            // TODO: define 'this' in symbol table
            // This requires moving a pointer not the value in 0
            // Review VM translator for details
        }

        self.compile_parameter_list()?;

        self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;

        // Local Var Declarations
        while let Ok(token) = self.peek() {
            if let TokenType::Segment(Segment::Var) = token.value {
                self.compile_var_dec()?;
            } else {
                break;
            }
        }

        //NOTE: Var is expected to carry it's offset count
        let n_locals = self.symbol_table.var_count(Segment::Var);
        self.writer.write_function(&full_name, n_locals);

        match routine_kind {
            Routine::Constructor => {
                let n_fields = self.symbol_table.var_count(Segment::Field);
                self.writer.write_push(Segment::Const, n_fields);
                self.writer.write_call("Memory.alloc", 1);
                self.writer.write_pop(Segment::Pointer, 0); // Set 'this'
            },
            Routine::Method => {
                self.writer.write_push(Segment::Arg, 0);    // Get 'this' (arg 0)
                self.writer.write_pop(Segment::Pointer, 0); // Set 'this' pointer
            },
            Routine::Function => {}, // Static function
        }

        self.compile_statements()?;
        // TODO: Flush local and Var from symbol table
        Ok(())
    }

    fn compile_parameter_list(&mut self) -> Result<(), String> {
        let mut token = self.advance()?;

        if let TokenType::Symbol(Symbol::Delim(Delimiter::CloseParen)) = token.value {
            return Ok(());
        }

        loop {
            let type_desc = match token.value {
                TokenType::Keywd(Keyword::Type(t)) => TypeDescriptor::Primitive(t),
                TokenType::Id(s) => TypeDescriptor::Class(s),
                _ => return Err(format!("{}: Expected type, found '{}'", token.line, token)),
            };

            let name = self.expect_ident()?;
            self.symbol_table.define_arg(&name, &type_desc);

            token = self.advance()?;
            match token.value {
                TokenType::Symbol(Symbol::Delim(Delimiter::CloseParen)) => return Ok(()),
                TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) => {
                    token = self.advance()?;
                },
                _ => return Err(format!("{}: Expected ',' or ')', found '{}'", token.line, token)),
            }
        }
    }

    fn compile_var_dec(&mut self) -> Result<(), String> {
        self.advance()?; // Eat 'var'
        let type_desc = self.get_type_descriptor()?;

        // Vectorized: Consume "x, y, z;" and define all of them
        let names = self.expect_ident_list()?;
        for name in names {
            self.symbol_table.define(&name, &type_desc, Segment::Var);
        }

        Ok(())
    }

    // --- Helper: Vectorized Identifier List ---

    // Consumes a list of identifiers "x, y, z;" and returns them as a Vec
    // Handles the comma separation and ensures the final semicolon is consumed
    fn expect_ident_list(&mut self) -> Result<Vec<String>, String> {
        let mut names = Vec::new();

        loop {
            let name = self.expect_ident()?;
            names.push(name);

            let token = self.advance()?;
            match token.value {
                TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) => {
                    continue;
                },
                TokenType::Symbol(Symbol::Delim(Delimiter::Semicolon)) => {
                    break;
                },
                // NOTE: Could be ',' but if an error is thrown the list was exited incorrectly
                _ => return Err(format!("{}: Expected ';' found '{}'", token, token.line)),
            }
        }
        Ok(names)
    }

    fn compile_statements(&mut self) -> Result<(), String> {
        // NOTE: Subroutines put variable declaration before statements so we don't expect '{'

        loop {
            let token = self.advance()?;

            match token.value {
                TokenType::Keywd(Keyword::Action(action)) => {
                    match action {
                        Action::Let => self.compile_let()?,
                        Action::If => self.compile_if()?,
                        Action::While => self.compile_while()?,
                        Action::Do => self.compile_do()?,
                        Action::Return => self.compile_return()?,
                        Action::Else => return Err(format!("{}: Unexpected 'else'", token.line)),
                    }
                },

                TokenType::Symbol(Symbol::Delim(Delimiter::CloseBrace)) => {
                    return Ok(()); // Exit this set of statements
                },

                _ => return Err(format!("{}: Expected statement or '}', found '{}'", token.line, token)),
            }
        }
    }

    fn compile_let(&mut self) -> Result<(), String> {
        let name = self.expect_ident()?;

        let mut is_array = false;
        if let TokenType::Symbol(Symbol::Delim(Delimiter::OpenBracket)) = self.peek()?.value {
            is_array = true;
            let kind = self.symbol_table.kind_of(&name).ok_or("Undefined var")?;
            let index = self.symbol_table.index_of(&name);
            self.writer.write_push(kind, index);

            self.advance()?; // [
            self.compile_expression()?;
            self.expect_symbol(Symbol::Delim(Delimiter::CloseBracket))?;

            self.writer.write_arithmetic(Operator::Add); // Base + Index
        }

        self.expect_symbol(Symbol::Op(Operator::Equal))?;
        self.compile_expression()?;
        self.expect_symbol(Symbol::Delim(Delimiter::Semicolon))?;

        if is_array {
            self.writer.write_pop(Segment::Temp, 0);    // Save Result
            self.writer.write_pop(Segment::Pointer, 1); // Set That = Address
            self.writer.write_push(Segment::Temp, 0);   // Restore Result
            self.writer.write_pop(Segment::That, 0);    // Store
        } else {
            let kind = self.symbol_table.kind_of(&name).ok_or("Undefined var")?;
            let index = self.symbol_table.index_of(&name);
            self.writer.write_pop(kind, index);
        }
        Ok(())
    }

    fn compile_if(&mut self) -> Result<(), String> {
        let l_true = format!("IF_TRUE{}", self.label_count);
        let l_false = format!("IF_FALSE{}", self.label_count);
        let l_end = format!("IF_END{}", self.label_count);
        self.label_count += 1;

        self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;
        self.compile_expression()?;
        self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;

        self.writer.write_if(&l_true);
        self.writer.write_goto(&l_false);
        self.writer.write_label(&l_true);

        self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;
        self.compile_statements()?;

        if let Ok(token) = self.peek() {
            if let TokenType::Keywd(Keyword::Action(Action::Else)) = token.value {
                self.writer.write_goto(&l_end);
                self.writer.write_label(&l_false);
                self.advance()?; // Else
                self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;
                self.compile_statements()?;
                self.writer.write_label(&l_end);
                return Ok(());
            }
        }
        self.writer.write_label(&l_false);
        Ok(())
    }

    fn compile_while(&mut self) -> Result<(), String> {
        let l_exp = format!("WHILE_EXP{}", self.label_count);
        let l_end = format!("WHILE_END{}", self.label_count);
        self.label_count += 1;

        self.writer.write_label(&l_exp);
        self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;
        self.compile_expression()?;
        self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;

        self.writer.write_not();
        self.writer.write_if(&l_end);

        self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;
        self.compile_statements()?;

        self.writer.write_goto(&l_exp);
        self.writer.write_label(&l_end);
        Ok(())
    }

    fn compile_do(&mut self) -> Result<(), String> {
        self.compile_expression()?;
        self.expect_symbol(Symbol::Delim(Delimiter::Semicolon))?;
        self.writer.write_pop(Segment::Temp, 0); // Dump return value
        Ok(())
    }

    fn compile_return(&mut self) -> Result<(), String> {
        if let TokenType::Symbol(Symbol::Delim(Delimiter::Semicolon)) = self.peek()?.value {
            self.writer.write_push(Segment::Const, 0);
        } else {
            self.compile_expression()?;
        }
        self.expect_symbol(Symbol::Delim(Delimiter::Semicolon))?;
        self.writer.write_return();
        Ok(())
    }

    fn compile_expression(&mut self) -> Result<(), String> {
        self.compile_term()?;

        loop {
            let token = self.peek()?;
            if let TokenType::Symbol(Symbol::Op(op)) = &token.value {
                if matches!(op, Operator::Not) { break; } // TODO: Throw error for notting AFTER

                let op_clone = op.clone();
                self.advance()?; // Eat op
                self.compile_term()?;

                match op_clone {
                    Operator::Mult => self.writer.write_call("Math.multiply", 2),
                    Operator::Divide => self.writer.write_call("Math.divide", 2),
                    _ => self.writer.write_arithmetic(op_clone),
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn compile_term(&mut self) -> Result<(), String> {
        let token = self.advance()?;
        match token.value {
            TokenType::IntConst(val) => self.writer.write_push(Segment::Const, val as usize),
            TokenType::StrgConst(val) => {
                self.writer.write_push(Segment::Const, val.len());
                self.writer.write_call("String.new", 1);
                for c in val.chars() {
                    self.writer.write_push(Segment::Const, c as usize);
                    self.writer.write_call("String.append", 2);
                }
            },
            TokenType::Keywd(Keyword::Const(k)) => match k {
                Const::True  => { self.writer.write_push(Segment::Const, 0); self.writer.write_not(); },
                Const::False | Const::Null => self.writer.write_push(Segment::Const, 0),
                Const::This  => self.writer.write_push(Segment::Pointer, 0),
            },
            TokenType::Symbol(Symbol::Op(op)) => {
                match op {
                    Operator::Sub => { self.compile_term()?; self.writer.write_neg(); },
                    // WARN: Tokenizer now parses negatives so this isn't necessary but we might
                    // break subtraction
                    Operator::Not => { self.compile_term()?; self.writer.write_not(); },
                    _ => return Err(format!("{}: Unexpected binary operator in unary position", token.line)),
                }
            },
            TokenType::Symbol(Symbol::Delim(Delimiter::OpenParen)) => {
                self.compile_expression()?;
                self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
            },
            TokenType::Id(name) => {
                match self.peek()?.value {
                    TokenType::Symbol(Symbol::Delim(Delimiter::OpenBracket)) => {
                        // Array: name[expr]
                        let kind = self.symbol_table.kind_of(&name).ok_or("Undefined array")?;
                        let index = self.symbol_table.index_of(&name);
                        self.writer.write_push(kind, index);
                        self.advance()?; // [
                        self.compile_expression()?;
                        self.expect_symbol(Symbol::Delim(Delimiter::CloseBracket))?;
                        self.writer.write_arithmetic(Operator::Add);
                        self.writer.write_pop(Segment::Pointer, 1);
                        self.writer.write_push(Segment::That, 0);
                    },
                    TokenType::Symbol(Symbol::Delim(Delimiter::OpenParen)) => {
                        // Method call on implicit this: name(args)
                        self.advance()?; // (
                        self.writer.write_push(Segment::Pointer, 0);
                        let n_args = self.compile_expression_list()?;
                        self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
                        self.writer.write_call(&format!("{}.{}", self.class_name, name), n_args + 1);
                    },
                    TokenType::Symbol(Symbol::Delim(Delimiter::Dot)) => {
                        // Explicit Call: class.func() or var.method()
                        self.advance()?; // .
                        let sub_name = self.expect_ident()?;
                        self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;


                        // TODO: Check this once symbol table is cleaned up
                        if let Some(kind) = self.symbol_table.kind_of(&name) {
                            let idx = self.symbol_table.index_of(&name);
                            self.writer.write_push(kind, idx);
                            let n_args = self.compile_expression_list()?;
                            self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
                            //TODO: Check this after writer
                            self.writer.write_call(&format!("TYPE_OF_{}.{}", name, sub_name), n_args + 1);
                        } else {
                            let n_args = self.compile_expression_list()?;
                            self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
                            self.writer.write_call(&format!("{}.{}", name, sub_name), n_args);
                        }
                    },
                    _ => {
                        // Simple Var
                        let kind = self.symbol_table.kind_of(&name).ok_or("Undefined var")?;
                        let index = self.symbol_table.index_of(&name);
                        self.writer.write_push(kind, index);
                    }
                }
            },
            _ => return Err(format!("{}: Unexpected term token: '{}'", token.line, token)),
        }
        Ok(())
    }

    fn compile_expression_list(&mut self) -> Result<usize, String> {
        let mut count = 0;
        if let TokenType::Symbol(Symbol::Delim(Delimiter::CloseParen)) = self.peek()?.value {
            return Ok(0);
        }
        loop {
            self.compile_expression()?;
            count += 1;
            // Since our expression list can end in a variety of characters we instead check the
            // list is well formed
            if let TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) = self.peek()?.value {
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(count)
    }

    fn peek(&mut self) -> Result<&Token, String> {
        match self.tokenizer.peek() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(e.clone()),
            None => Err("Unexpected EOF".to_string()),
        }
    }

    fn advance(&mut self) -> Result<Token, String> {
        match self.tokenizer.next() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(e),
            None => Err("Unexpected EOF".to_string()),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        let token = self.advance()?;
        match token.value {
            TokenType::Id(s) => Ok(s),
            _ => Err(format!("{}: Expected identifier, found '{}'", token.line, token)),
        }
    }

    fn expect_symbol(&mut self, expected: Symbol) -> Result<(), String> {
        let token = self.advance()?;
        match token.value {
            TokenType::Symbol(ref s) if *s == expected => Ok(()),
            _ => Err(format!("{}: Expected symbol '{}', found '{}'", token.line, expected, token)),
        }
    }

    fn get_type_descriptor(&mut self) -> Result<TypeDescriptor, String> {
        let token = self.advance()?;
        match token.value {
            TokenType::Keywd(Keyword::Type(t)) => Ok(TypeDescriptor::Primitive(t)),
            TokenType::Id(s) => Ok(TypeDescriptor::Class(s)),
            _ => Err(format!("{}: Expected type (int, char, boolean, ClassName), found '{}'", token, token.line)),
        }
    }
}
