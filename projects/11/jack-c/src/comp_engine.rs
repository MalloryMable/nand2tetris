use std::iter::Peekable;
use crate::tokenizer::JackTokenizer;
use crate::tokens::{
    Token, TokenType, Keyword, Symbol, Delimiter, Operator,
    Segment, Action, Routine, Const, Primitive
};
use crate::symbol_table::{ TypeDescriptor, SymbolTable};

pub struct CompilationEngine<'a, W: VMWriter> {
    tokenizer: Peekable<JackTokenizer<'a>>,
    writer: &'a mut W,
    // TODO: Pass in registry here
    symbol_table: SymbolTable,
    class_name: String,
    label_count: usize,
}

impl<'a, W: VMWriter> CompilationEngine<'a, W> {
    pub fn new(tokenizer: JackTokenizer<'a>, writer: &'a mut W) -> Self {
        Self {
            tokenizer: tokenizer.peekable(),
            writer,
            symbol_table: SymbolTable::new(),
            class_name: String::new(),
            label_count: 0,
        }
    }

    pub fn compile_class(&mut self) -> Result<(), String> {
        let token = self.advance()?;
        match token.value {
            TokenType::Keywd(Keyword::Class) => {},
            _ => return Err(format!("{}: Expected 'class' declaration, found '{}'", token.line, token)),
        }
        self.class_name = self.expect_id()?;
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
        // Symbol table is discarded here implicitly
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
        let names = self.expect_id_list()?;
        for name in names {
            self.symbol_table.define(&name, &type_desc, segment.clone());
        }

        Ok(())
    }

    fn compile_subroutine(&mut self) -> Result<(), String> {
        self.symbol_table.start_subroutine();


        // TODO: Update when doing writer
        if let TokenType::Keywd(Keyword::Routine(routine)) = token.value {
            match routine {
                Routine::Constructor => {
                    // Constructors have an implicit return type and name
                    let func_name = if self.expect_id()? == self.class_name {
                        format!("{}.new", self.class_name)
                    } else {
                        return Err(format!("{}: Constructor name must match class name"));
                    };

                    self.compile_parameter_list()?;

                    self.writer.write_function(&func_name, self.symbol_table.arg_count);

                    // We set aside the number of required fields for the new object
                    self.writer.write_push(Segment::Const, self.symbol_table.field_count);
                    self.writer.write_call("Memory.alloc", 1);
                    // Set the base segment for the memory alocated to the current object
                    self.writer.write_pop(Segment::Pointer, 0); // Set 'this'
                },
                Routine::Method => {
                    // return type is collected but we don't do a lot of type checking in this
                    let _routine_return_type = self.get_type_descriptor()?;
                    let func_name = format!("{}.{}", self.class_name, self.expect_id()?);

                    self.compile_parameter_list()?;

                    // Last argument we push into the frame before the function call
                    self.writer.write_pop(Segment::Pointer, 0); // Set 'this' pointer

                    self.writer.write_function(&func_name,
                        1 + self.symbol_table.arg_count);

                    // TODO: make sure the first arg pushed on mthd calls is the id's pointer
               },
                Routine::Function => {
                    let _routine_return_type = self.get_type_descriptor()?;
                    let func_name = format!("{}.{}", self.class_name, self.expect_id()?);

                    self.compile_parameter_list()?;

                   self.writer.write_function(&func_name, self.symbol_table.arg_count);
                }, // Static function
                _ => return Err(format!("{}: Expected constructor, function, or method, found '{}'", token.line, token)),
            }
        }


        // Now that we know how many args to take we call the function
        // NOTE: to make function calling type sensitive we return and pass around an array of
        match routine_kind {
                    }

        self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;

        // Local Var Declarations
        while let Ok(token) = self.peek() {
            if let TokenType::Segment(Segment::Var) = token.value {
                self.compile_var_dec()?;
            } else {
                break;
            }
        }

        self.compile_statements()?;
        // TODO: Flush local and Var from symbol table
        Ok(())
    }

    // objects
    fn compile_parameter_list(&mut self) -> Result<(), String> {
        self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;

        token = self.advance()?;
        if let TokenType::Symbol(Symbol::Delim(Delimiter::CloseParen)) = token.value {
            return Ok(());
        }

        loop {
            let type_desc = match token.value {
                TokenType::Keywd(Keyword::Prim(t)) => TypeDescriptor::Primitive(t),
                TokenType::Id(s) => TypeDescriptor::Class(s),
                _ => return Err(format!("{}: Expected type, found '{}'", token.line, token)),
            };

            let name = self.expect_id()?;
            self.symbol_table.define(&name, &type_desc, Segment::Arg);

            token = self.advance()?;
            match token.value {
                TokenType::Symbol(Symbol::Delim(Delimiter::CloseParen)) => return Ok(()),
                TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) => {
                    token = self.advance()?;
                },
                _ => return Err(format!("{}: Expected ')', found '{}'", token.line, token)),
            }
        }
    }

    fn compile_var_dec(&mut self) -> Result<(), String> {
        self.advance()?; // Eat 'var'
        let type_desc = self.get_type_descriptor()?;

        // Vectorized: Consume "x, y, z;" and define all of them
        for name in self.expect_id_list()? {
            self.symbol_table.define(&name, &type_desc, Segment::Var);
        }

        Ok(())
    }

    fn expect_id_list(&mut self) -> Result<Vec<String>, String> {
        let mut names = Vec::new();

        loop {
            let name = self.expect_id()?;
            names.push(name);

            let token = self.advance()?;
            match token.value {
                TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) => {
                    continue;
                },
                TokenType::Symbol(Symbol::Delim(Delimiter::Semicolon)) => {
                    break;
                },
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
        let name = self.expect_id()?;

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
                // NOTE: If we made this brace optional we could chain else- if satements
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

        while let Ok(token) = self.peek() {
            if let TokenType::Symbol(Symbol::Op(op)) = &token.value {
                if matches!(op, Operator::Not) {
                    return Err(format!("{}: 'Not' operator cannot be used as a binary operator", token.line));
                }

                let op_clone = op.clone();
                self.advance()?; // Eat op
                self.compile_term()?; // Second variable is pushed to the stack

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
            TokenType::IntConst(val) => self.writer.write_push(Segment::Const, val),
            TokenType::StrgConst(val) => {
                self.writer.write_push(Segment::Const, val.len());
                self.writer.write_call("String.new", 1);
                for c in val.chars() {
                    self.writer.write_push(Segment::Const, c as usize); // Cast char to int
                    self.writer.write_call("String.append", 2);
                }
            },
            TokenType::Keywd(k) => match k {
                Keyword::Const(Const::True) => {
                    self.writer.write_push(Segment::Const, 0);
                    self.writer.write_not(); // True is -1 (bitwise not of 0)
                },
                Keyword::Const(Const::False) | Keyword::Const(Const::Null) => {
                    self.writer.write_push(Segment::Const, 0);
                },
                Keyword::Const(Const::This) => self.writer.write_push(Segment::Pointer, 0),
                _ => return Err(format!("{}: Expected constant, found keyword '{}'", token.line, k)),
            },
            TokenType::Symbol(Symbol::Op(op)) => {
                // Unary Operators (-term, ~term)
                match op {
                    Operator::Sub => { self.compile_term()?; self.writer.write_neg(); },
                    Operator::Not => { self.compile_term()?; self.writer.write_not(); },
                    _ => return Err(format!("{}: Unexpected binary operator '{}' in unary position", token.line, op)),
                }
            },
            TokenType::Symbol(Symbol::Delim(Delimiter::OpenParen)) => {
                self.compile_expression()?;
                self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
            },
            TokenType::Id(name) => {
                // Delegate to helper, pass ownership of name
                self.compile_id(name)?;
            },
            _ => return Err(format!("{}: Unexpected term token: '{}'", token.line, token)),
        }
        Ok(())
    }

    // Helper for Identifier Terms (Vars, Arrays, Calls)
    fn compile_id(&mut self, name: String) -> Result<(), String> {
        let next_token = self.peek().map_err(|e| e.to_string())?;

        match next_token.value {
            // Array Access: a[i]
            TokenType::Symbol(Symbol::Delim(Delimiter::OpenBracket)) => {
                let kind = self.symbol_table.kind_of(&name)
                    .ok_or_else(|| format!("Undefined array: {}", name))?;
                let index = self.symbol_table.index_of(&name)
                    .ok_or_else(|| format!("Index missing for: {}", name))?;

                self.writer.write_push(kind, index); // Push Base
                self.advance()?; // Eat [
                self.compile_expression()?; // Push Index
                self.expect_symbol(Symbol::Delim(Delimiter::CloseBracket))?;

                self.writer.write_arithmetic(Operator::Add); // Base + Index
                self.writer.write_pop(Segment::Pointer, 1);  // Set That = Address
                self.writer.write_push(Segment::That, 0);    // Push *Address
            },
            // Method Call: foo() -> implicit this.foo()
            TokenType::Symbol(Symbol::Delim(Delimiter::OpenParen)) => {
                self.advance()?; // Eat (
                self.writer.write_push(Segment::Pointer, 0); // Push 'this'
                let n_args = self.compile_expression_list()?;
                self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;

                self.writer.write_call(&format!("{}.{}", self.class_name, name), n_args + 1);
            },
            // Dot Access: obj.method() or Class.func()
            TokenType::Symbol(Symbol::Delim(Delimiter::Dot)) => {
                self.advance()?; // Eat .
                let sub_name = self.expect_id()?;
                self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;

                if let Some(kind) = self.symbol_table.kind_of(&name) {
                    // Instance Method: var.method()
                    let index = self.symbol_table.index_of(&name).unwrap();
                    self.writer.write_push(kind, index); // Push Instance

                    let type_desc = self.symbol_table.type_of(&name).unwrap();
                    let class_name = match type_desc {
                        TypeDescriptor::Class(c) => c,
                        _ => return Err(format!("Cannot call method on primitive {}", name)),
                    };

                    let n_args = self.compile_expression_list()?;
                    self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
                    self.writer.write_call(&format!("{}.{}", class_name, sub_name), n_args + 1);
                } else {
                    // Static Function: Class.func()
                    let n_args = self.compile_expression_list()?;
                    self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
                    self.writer.write_call(&format!("{}.{}", name, sub_name), n_args);
                }
            },
            // Simple Variable
            _ => {
                let kind = self.symbol_table.kind_of(&name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))?;
                let index = self.symbol_table.index_of(&name)
                    .ok_or_else(|| format!("Index missing for: {}", name))?;
                self.writer.write_push(kind, index);
            }
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

    fn expect_id(&mut self) -> Result<String, String> {
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
