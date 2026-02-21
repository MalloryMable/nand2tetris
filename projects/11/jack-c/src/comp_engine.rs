

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
        match self.advance()?.value {
            TokenType::Identifier(s) => Ok(s),
            t => Err(format!("Expected identifier, found {:?}", t)),
        }
    }

    fn expect_symbol(&mut self, expected: Symbol) -> Result<(), String> {
        match self.advance()?.value {
            TokenType::Symbol(s) if s == expected => Ok(()),
            t => Err(format!("Expected symbol {:?}, found {:?}", expected, t)),
        }
    }

    // Retrieves a type which is either a primitive keyword (int/char) or an Identifier (ClassName)
    fn get_type_descriptor(&mut self) -> Result<TypeDescriptor, String> {
        let token = self.advance()?;
        match token.value {
            TokenType::Keyword(Keyword::Type(t)) => Ok(TypeDescriptor::Primitive(t)),
            TokenType::Identifier(s) => Ok(TypeDescriptor::Class(s)),
            _ => Err(format!("Expected type (int, char, boolean, ClassName), found {:?}", token)),
        }
    }

    // --- Compilation Logic ---

    pub fn compile_class(&mut self) -> Result<(), String> {
        match self.advance()?.value {
            TokenType::Keyword(Keyword::Class) => {},
            _ => return Err("Expected 'class' declaration".to_string()),
        }
        self.class_name = self.expect_ident()?;
        self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;

        // Class Variable Declarations
        while let Ok(token) = self.peek() {
            if let TokenType::Keyword(Keyword::Segment(seg)) = &token.value {
                // Ensure it's static or field (Var is for subroutines)
                match seg {
                    KwSegment::Static | KwSegment::Field => self.compile_class_var_dec()?,
                    _ => break,
                }
            } else {
                break;
            }
        }

        // Subroutine Declarations
        while let Ok(token) = self.peek() {
            if let TokenType::Keyword(Keyword::Routine(_)) = &token.value {
                self.compile_subroutine()?;
            } else {
                break;
            }
        }

        self.expect_symbol(Symbol::Delim(Delimiter::CloseBrace))?;
        Ok(())
    }

    fn compile_class_var_dec(&mut self) -> Result<(), String> {
        let segment = match self.advance()?.value {
            TokenType::Keyword(Keyword::Segment(s)) => s,
            _ => return Err("Expected static or field".to_string()),
        };

        let type_desc = self.get_type_descriptor()?;

        loop {
            let name = self.expect_ident()?;
            self.symbol_table.define(&name, &type_desc, segment.clone());

            match self.peek()?.value {
                TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) => { self.advance()?; },
                _ => break,
            }
        }
        self.expect_symbol(Symbol::Delim(Delimiter::Semicolon))?;
        Ok(())
    }

    fn compile_subroutine(&mut self) -> Result<(), String> {
        self.symbol_table.start_subroutine();

        let routine_kind = match self.advance()?.value {
            TokenType::Keyword(Keyword::Routine(r)) => r,
            _ => return Err("Expected constructor, function, or method".to_string()),
        };

        let routine_return_type = self.advance()?;

        // 3. Name
        let func_name = self.expect_ident()?;
        let full_name = format!("{}.{}", self.class_name, func_name);

        self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;

        // Implicit 'this' for methods
        if let KwRoutine::Method = routine_kind {
            // TODO: Needs symbol table
        }

        self.compile_parameter_list()?;
        self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;

        self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;

        // 4. Local Var Declarations
        while let Ok(token) = self.peek() {
            // Strictly match 'var' segment
            if let TokenType::Keyword(Keyword::Segment(KwSegment::Var)) = token.value {
                self.compile_var_dec()?;
            } else {
                break;
            }
        }

        let n_locals = self.symbol_table.var_count(KwSegment::Var);
        self.writer.write_function(&full_name, n_locals);

        match routine_kind {
            KwRoutine::Constructor => {
                let n_fields = self.symbol_table.var_count(KwSegment::Field);
                self.writer.write_push(SegmentKind::Const, n_fields);
                self.writer.write_call("Memory.alloc", 1);
                self.writer.write_pop(SegmentKind::Pointer, 0); // Set 'this'
            },
            KwRoutine::Method => {
                self.writer.write_push(SegmentKind::Arg, 0);    // Get 'this' (arg 0)
                self.writer.write_pop(SegmentKind::Pointer, 0); // Set 'this' pointer
            },
            KwRoutine::Function => {}, // Static function, no setup needed
        }

        self.compile_statements()?;
        self.expect_symbol(Symbol::Delim(Delimiter::CloseBrace))?;
        Ok(())
    }

    fn compile_parameter_list(&mut self) -> Result<(), String> {
        if let Ok(token) = self.peek() {
            if let TokenType::Symbol(Symbol::Delim(Delimiter::CloseParen)) = token.value {
                return Ok(());
            }
        }
        loop {
            let type_desc = self.get_type_descriptor()?;
            let name = self.expect_ident()?;
            self.symbol_table.define_arg(&name, &type_desc);

            if let TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) = self.peek()?.value {
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn compile_var_dec(&mut self) -> Result<(), String> {
        self.advance()?; // Eat 'var'
        let type_desc = self.get_type_descriptor()?;
        loop {
            let name = self.expect_ident()?;
            self.symbol_table.define(&name, &type_desc, KwSegment::Var);

            if let TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) = self.peek()?.value {
                self.advance()?;
            } else {
                break;
            }
        }
        self.expect_symbol(Symbol::Delim(Delimiter::Semicolon))?;
        Ok(())
    }


    fn compile_statements(&mut self) -> Result<(), String> {
        loop {
            let token = match self.peek() { Ok(t) => t, _ => break };

            // Only match Action keywords
            if let TokenType::Keyword(Keyword::Action(action)) = &token.value {
                match action {
                    KwAction::Let    => self.compile_let()?,
                    KwAction::If     => self.compile_if()?,
                    KwAction::While  => self.compile_while()?,
                    KwAction::Do     => self.compile_do()?,
                    KwAction::Return => self.compile_return()?,
                    KwAction::Else   => break, // Should be handled by If
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn compile_let(&mut self) -> Result<(), String> {
        self.advance()?; // Let
        let name = self.expect_ident()?;

        let mut is_array = false;
        if let TokenType::Symbol(Symbol::Delim(Delimiter::OpenBracket)) = self.peek()?.value {
            is_array = true;
            // Array Access logic: push arr, push index, add
            let kind = self.symbol_table.kind_of(&name).ok_or("Undefined var")?;
            let index = self.symbol_table.index_of(&name);
            self.writer.write_push(SegmentKind::from(kind), index);

            self.advance()?; // [
            self.compile_expression()?;
            self.expect_symbol(Symbol::Delim(Delimiter::CloseBracket))?;

            self.writer.write_arithmetic(Operator::Plus); // Base + Index
        }

        self.expect_symbol(Symbol::Op(Operator::Equal))?;
        self.compile_expression()?;
        self.expect_symbol(Symbol::Delim(Delimiter::Semicolon))?;

        if is_array {
            self.writer.write_pop(SegmentKind::Temp, 0);    // Save Result
            self.writer.write_pop(SegmentKind::Pointer, 1); // Set That = Address
            self.writer.write_push(SegmentKind::Temp, 0);   // Restore Result
            self.writer.write_pop(SegmentKind::That, 0);    // Store
        } else {
            let kind = self.symbol_table.kind_of(&name).ok_or("Undefined var")?;
            let index = self.symbol_table.index_of(&name);
            self.writer.write_pop(SegmentKind::from(kind), index);
        }
        Ok(())
    }

    fn compile_if(&mut self) -> Result<(), String> {
        self.advance()?; // If
        // TODO: Check against translator logic
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
        self.expect_symbol(Symbol::Delim(Delimiter::CloseBrace))?;

        if let Ok(token) = self.peek() {
            if let TokenType::Keyword(Keyword::Action(KwAction::Else)) = token.value {
                self.writer.write_goto(&l_end);
                self.writer.write_label(&l_false);
                self.advance()?; // Else
                self.expect_symbol(Symbol::Delim(Delimiter::OpenBrace))?;
                self.compile_statements()?;
                self.expect_symbol(Symbol::Delim(Delimiter::CloseBrace))?;
                self.writer.write_label(&l_end);
                return Ok(());
            }
        }
        self.writer.write_label(&l_false);
        Ok(())
    }

    fn compile_while(&mut self) -> Result<(), String> {
        self.advance()?; // While
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
        self.expect_symbol(Symbol::Delim(Delimiter::CloseBrace))?;

        self.writer.write_goto(&l_exp);
        self.writer.write_label(&l_end);
        Ok(())
    }

    fn compile_do(&mut self) -> Result<(), String> {
        self.advance()?; // Do
        self.compile_expression()?; // This handles the func call
        self.expect_symbol(Symbol::Delim(Delimiter::Semicolon))?;
        self.writer.write_pop(SegmentKind::Temp, 0); // Dump return value
        Ok(())
    }

    fn compile_return(&mut self) -> Result<(), String> {
        self.advance()?; // Return
        if let TokenType::Symbol(Symbol::Delim(Delimiter::Semicolon)) = self.peek()?.value {
            self.writer.write_push(SegmentKind::Const, 0);
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
                // Check if it is a binary operator (Tilde is unary)
                if matches!(op, Operator::Tilde) { break; }

                // TODO: Check why we do this
                let op_clone = op.clone();
                self.advance()?; // Eat op
                self.compile_term()?;

                // Direct Enum dispatch for Math
                match op_clone {
                    Operator::Mult => self.writer.write_call("Math.multiply", 2),
                    Operator::Slash => self.writer.write_call("Math.divide", 2),
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
            TokenType::IntConst(val) => self.writer.write_push(SegmentKind::Const, val as usize),
            TokenType::StringConst(val) => {
                self.writer.write_push(SegmentKind::Const, val.len());
                self.writer.write_call("String.new", 1);
                for c in val.chars() {
                    self.writer.write_push(SegmentKind::Const, c as usize);
                    self.writer.write_call("String.append", 2);
                }
            },
            // Constants Matching
            TokenType::Keyword(Keyword::Const(k)) => match k {
                KwConst::True  => { self.writer.write_push(SegmentKind::Const, 0); self.writer.write_not(); },
                KwConst::False | KwConst::Null => self.writer.write_push(SegmentKind::Const, 0),
                KwConst::This  => self.writer.write_push(SegmentKind::Pointer, 0),
            },
            // Unary Ops
            TokenType::Symbol(Symbol::Op(op)) => {
                match op {
                    Operator::Minus => { self.compile_term()?; self.writer.write_neg(); },
                    Operator::Tilde => { self.compile_term()?; self.writer.write_not(); },
                    _ => return Err("Unexpected binary operator in unary position".to_string()),
                }
            },
            // Grouping
            TokenType::Symbol(Symbol::Delim(Delimiter::OpenParen)) => {
                self.compile_expression()?;
                self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
            },
            // Identifiers (Var, Array, Call)
            TokenType::Identifier(name) => {
                match self.peek()?.value {
                    TokenType::Symbol(Symbol::Delim(Delimiter::OpenBracket)) => {
                        // Array: name[expr]
                        let kind = self.symbol_table.kind_of(&name).ok_or("Undefined array")?;
                        let index = self.symbol_table.index_of(&name);
                        self.writer.write_push(SegmentKind::from(kind), index);
                        self.advance()?; // [
                        self.compile_expression()?;
                        self.expect_symbol(Symbol::Delim(Delimiter::CloseBracket))?;
                        self.writer.write_arithmetic(Operator::Plus);
                        self.writer.write_pop(SegmentKind::Pointer, 1);
                        self.writer.write_push(SegmentKind::That, 0);
                    },
                    TokenType::Symbol(Symbol::Delim(Delimiter::OpenParen)) => {
                        // Method call on implicit this: name(args)
                        self.advance()?; // (
                        self.writer.write_push(SegmentKind::Pointer, 0);
                        let n_args = self.compile_expression_list()?;
                        self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
                        self.writer.write_call(&format!("{}.{}", self.class_name, name), n_args + 1);
                    },
                    TokenType::Symbol(Symbol::Delim(Delimiter::Dot)) => {
                        // Explicit Call: class.func() or var.method()
                        self.advance()?; // .
                        let sub_name = self.expect_ident()?;
                        self.expect_symbol(Symbol::Delim(Delimiter::OpenParen))?;

                        // Check symbol table to see if 'name' is a variable (Instance method) or Class (Static)
                        if let Some(kind) = self.symbol_table.kind_of(&name) {
                            let idx = self.symbol_table.index_of(&name);
                            self.writer.write_push(SegmentKind::from(kind), idx);
                            // Assume variable type name is resolved properly in real impl
                            let n_args = self.compile_expression_list()?;
                            self.expect_symbol(Symbol::Delim(Delimiter::CloseParen))?;
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
                        self.writer.write_push(SegmentKind::from(kind), index);
                    }
                }
            },
            _ => return Err(format!("Unexpected term token: {:?}", token)),
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
            if let TokenType::Symbol(Symbol::Delim(Delimiter::Comma)) = self.peek()?.value {
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(count)
    }
}
