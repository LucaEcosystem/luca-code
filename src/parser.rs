use crate::{
    ast::{BinaryOp, Block, CompareOp, ContainsMode, DictEntryExpr, Expr, LengthArg, Param, Program, RaiseArg, Statement, UnaryOp},
    error::LucaError,
    lexer::{Token, TokenKind},
    types::{CollectionKind, Type, Value},
};

pub fn parse(tokens: Vec<Token>) -> Result<Program, LucaError> { Parser { tokens, current: 0 }.program() }

struct Parser { tokens: Vec<Token>, current: usize }

impl Parser {
    fn program(mut self) -> Result<Program, LucaError> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::Eof) {
            if self.take(&TokenKind::Newline) { continue; }
            if self.take(&TokenKind::Dedent) {
                return Err(self.error("Unexpected dedent"));
            }
            if self.take(&TokenKind::Indent) {
                return Err(self.error("Unexpected indent"));
            }
            statements.push(self.statement()?);
        }
        Ok(Program { statements })
    }

    fn statement(&mut self) -> Result<Statement, LucaError> {
        let token = self.peek().clone();
        if self.take(&TokenKind::Def) {
            if self.take(&TokenKind::Func) {
                return self.func_def(token.line, token.column);
            }
            if self.check(&TokenKind::List) || self.check(&TokenKind::Tuple) || self.check(&TokenKind::Set) || self.check(&TokenKind::Dict) {
                let kind_token = self.advance().clone();
                let kind = match kind_token.kind {
                    TokenKind::List => CollectionKind::List,
                    TokenKind::Tuple => CollectionKind::Tuple,
                    TokenKind::Set => CollectionKind::Set,
                    TokenKind::Dict => CollectionKind::Dict,
                    _ => unreachable!(),
                };
                let (name, _, _) = self.identifier("Expected a collection name")?;
                let element_type = self.optional_type_assertion()?;
                self.consume(&TokenKind::Equal, "Expected '=' before the value")?;
                let value = self.expression()?;
                return Ok(Statement::DeclareCollection { kind, name, element_type, value, line: token.line, column: token.column });
            }
            if self.take(&TokenKind::Error) {
                let (name, _, _) = self.identifier("Expected error name after 'def error'")?;
                self.consume(&TokenKind::Colon, "Expected ':' after error name")?;
                let body = self.block()?;
                return Ok(Statement::ErrorDef { name, body, line: token.line, column: token.column });
            }
            let mutable = if self.take(&TokenKind::Var) { true } else if self.take(&TokenKind::Const) { false } else { return Err(self.error("Expected 'var', 'const', 'func', 'list', 'tuple', 'set', 'dict', or 'error' after 'def'")); };
            let (name, _, _) = self.identifier("Expected a declaration name")?;
            let declared_type = self.optional_type_assertion()?;
            // If declared_type is None, type will be inferred from value
            self.consume(&TokenKind::Equal, "Expected '=' before the value")?;
            let value = self.expression()?;
            return Ok(Statement::Declare { name, declared_type, mutable, value, line: token.line, column: token.column });
        }
        if self.take(&TokenKind::Return) {
            let line = token.line;
            let column = token.column;
            // Bare return or return with value: check if next token can start an expression
            // If next is newline, dedent, eof, or else/clear etc., then bare return
            if self.check(&TokenKind::Newline) || self.check(&TokenKind::Dedent) || self.check(&TokenKind::Eof) {
                return Ok(Statement::Return { value: None, line, column });
            }
            // Try to parse expression if possible, but don't consume if it's not an expression start
            // Peek if next is an expression start: literals, Identifier, LeftParen/LeftBracket/LeftBrace, Minus, Not
            let is_expr_start = matches!(
                self.peek().kind,
                TokenKind::Int(_)
                    | TokenKind::Dec(_)
                    | TokenKind::String(_)
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::None
                    | TokenKind::Identifier(_)
                    | TokenKind::LeftParen
                    | TokenKind::LeftBracket
                    | TokenKind::LeftBrace
                    | TokenKind::Minus
                    | TokenKind::Not
                    | TokenKind::Ask
            );
            if is_expr_start {
                let value = self.expression()?;
                return Ok(Statement::Return { value: Some(value), line, column });
            } else {
                return Ok(Statement::Return { value: None, line, column });
            }
        }
        if self.take(&TokenKind::Try) {
            return self.try_statement(token.line, token.column);
        }
        if self.take(&TokenKind::If) {
            return self.if_statement(token.line, token.column);
        }
        if self.take(&TokenKind::Import) {
            let (name, _, _) = self.identifier("Expected module name after 'import'")?;
            return Ok(Statement::Import { name, line: token.line, column: token.column });
        }
        if self.take(&TokenKind::For) {
            return self.for_statement(token.line, token.column);
        }
        if self.take(&TokenKind::While) {
            return self.while_statement(token.line, token.column);
        }
        if self.take(&TokenKind::Stop) {
            return Ok(Statement::Stop { line: token.line, column: token.column });
        }
        if self.take(&TokenKind::Skip) {
            return Ok(Statement::Skip { line: token.line, column: token.column });
        }
        if self.take(&TokenKind::Forget) {
            let is_const = if self.take(&TokenKind::Const) {
                true
            } else if self.take(&TokenKind::Var)
                || self.take(&TokenKind::List)
                || self.take(&TokenKind::Tuple)
                || self.take(&TokenKind::Set)
                || self.take(&TokenKind::Dict)
            {
                false
            } else {
                return Err(self.error("Expected 'var', 'const', 'list', 'tuple', 'set', or 'dict' after 'forget'"));
            };
            let (name, _, _) = self.identifier("Expected a name after 'forget'")?;
            let asserted_type = self.optional_type_assertion()?;
            return Ok(Statement::Forget { name, is_const, asserted_type, line: token.line, column: token.column });
        }
        if self.take(&TokenKind::Change) {
            // Optional var/const keyword after change, but primarily var
            if self.check(&TokenKind::Var) || self.check(&TokenKind::Const) {
                self.advance();
            }
            let (name, _, _) = self.identifier("Expected a variable name after 'change'")?;
            // Collection item update: change name[index] = value
            if self.check(&TokenKind::LeftBracket) {
                self.consume(&TokenKind::LeftBracket, "Expected '[' after collection name")?;
                let index = self.expression()?;
                self.consume(&TokenKind::RightBracket, "Expected ']' after index")?;
                self.consume(&TokenKind::Equal, "Expected '=' after ']' in 'change'")?;
                let value = self.expression()?;
                return Ok(Statement::ChangeIndex { name, index, value, line: token.line, column: token.column });
            }
            let asserted_type = self.optional_type_assertion()?;
            let new_type = if let Some(t) = asserted_type {
                t
            } else {
                return Err(self.error("Expected a type annotation in parentheses after 'change'"));
            };
            self.consume(&TokenKind::Equal, "Expected '=' after type annotation in 'change'")?;
            let value = self.expression()?;
            return Ok(Statement::Change { name, new_type, value, line: token.line, column: token.column });
        }
        if self.take(&TokenKind::Print) {
            self.consume(&TokenKind::LeftParen, "Expected '(' after print")?;
            let value = self.expression()?;
            self.consume(&TokenKind::RightParen, "print accepts exactly one argument")?;
            return Ok(Statement::Print { value, line: token.line, column: token.column });
        }
        if let TokenKind::Identifier(name) = token.kind.clone() {
            self.advance();
            // Check for method call like x.clear(), x.add(...), x.remove(...)
            if self.take(&TokenKind::Dot) {
                let (method, _, _) = self.identifier("Expected method name after '.'")?;
                if method == "clear" {
                    self.consume(&TokenKind::LeftParen, "Expected '(' after 'clear'")?;
                    self.consume(&TokenKind::RightParen, "Expected ')' after 'clear'")?;
                    return Ok(Statement::Clear { name, line: token.line, column: token.column });
                } else if method == "add" {
                    self.consume(&TokenKind::LeftParen, "Expected '(' after 'add'")?;
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            args.push(self.expression()?);
                            if self.take(&TokenKind::Comma) {
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                    return Ok(Statement::CollectionAdd { name, args, line: token.line, column: token.column });
                } else if method == "remove" {
                    self.consume(&TokenKind::LeftParen, "Expected '(' after 'remove'")?;
                    let arg = self.expression()?;
                    self.consume(&TokenKind::RightParen, "Expected ')' after argument")?;
                    return Ok(Statement::CollectionRemove { name, arg, line: token.line, column: token.column });
                } else {
                    // Generic member call statement: Module.func(args) — resolved at runtime
                    self.consume(&TokenKind::LeftParen, "Expected '(' after member name")?;
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            args.push(self.expression()?);
                            if self.take(&TokenKind::Comma) {
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                    let object = Expr::Variable { name, asserted_type: None, line: token.line, column: token.column };
                    return Ok(Statement::MemberCall { object: Box::new(object), member: method, args, line: token.line, column: token.column });
                }
            }
            // Bare index assignment without change is not allowed
            if self.check(&TokenKind::LeftBracket) {
                return Err(LucaError::new("Use 'change name[index] = value' to update collection items", token.line, token.column));
            }
            // Check for standalone call like foo() or foo(a, b)
            if self.check(&TokenKind::LeftParen) {
                // Determine if this is a type assertion for assignment vs a call
                // Look ahead: if '(' contains exactly one type name and then ')' and then '=' then it's assignment with type
                let is_assignment_with_type = if self.current + 2 < self.tokens.len() {
                    if let TokenKind::Identifier(type_name) = &self.tokens[self.current + 1].kind {
                        if Type::parse(type_name).is_some() && self.tokens[self.current + 2].kind == TokenKind::RightParen {
                            // Check if after ')' the next is '=' (assignment)
                            self.tokens.get(self.current + 3).is_some_and(|t| t.kind == TokenKind::Equal)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !is_assignment_with_type {
                    // Parse as call: '(' args ')'
                    self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            args.push(self.expression()?);
                            if self.take(&TokenKind::Comma) {
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                    // Standalone call statement should be alone on line (next is newline/dedent/eof), not assignment
                    // If next is '=', then this was actually an assignment with type, but we already handled that case above
                    // So this is a call statement
                    return Ok(Statement::Call { name, args, line: token.line, column: token.column });
                }
            }
            let asserted_type = self.optional_type_assertion()?;
            self.consume(&TokenKind::Equal, "Expected '=' in assignment")?;
            let value = self.expression()?;
            return Ok(Statement::Assign { name, asserted_type, value, line: token.line, column: token.column });
        }
        if self.take(&TokenKind::Raise) {
            let (name, _, _) = self.identifier("Expected error name after 'raise'")?;
            self.consume(&TokenKind::LeftParen, "Expected '(' after error name")?;
            let mut overrides = Vec::new();
            if !self.check(&TokenKind::RightParen) {
                loop {
                    let (field, field_line, field_column) = self.identifier("Expected field name")?;
                    self.consume(&TokenKind::Equal, "Expected '=' after field name")?;
                    let value = self.expression()?;
                    overrides.push(RaiseArg { field, value, line: field_line, column: field_column });
                    if self.take(&TokenKind::Comma) {
                        if self.check(&TokenKind::RightParen) {
                            break;
                        }
                        continue;
                    } else {
                        break;
                    }
                }
            }
            self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
            return Ok(Statement::Raise { name, overrides, line: token.line, column: token.column });
        }
        Err(self.error("Expected a declaration, assignment, print, return, if, for, while, stop, skip, import, try, or raise statement"))
    }

    fn func_def(&mut self, line: usize, column: usize) -> Result<Statement, LucaError> {
        // Already consumed 'def' and 'func', now expect name
        let (name, _, _) = self.identifier("Expected function name after 'def func'")?;
        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        // Handle empty params: immediate ')'
        if !self.check(&TokenKind::RightParen) {
            loop {
                let (param_name, p_line, p_col) = self.identifier("Expected parameter name")?;
                let ty = self.optional_type_assertion()?;
                let default = if self.take(&TokenKind::Equal) {
                    Some(self.expression()?)
                } else {
                    None
                };
                params.push(Param { name: param_name, ty, default, line: p_line, column: p_col });
                if self.take(&TokenKind::Comma) {
                    continue;
                } else {
                    break;
                }
            }
        }
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;
        let return_type = self.optional_type_assertion()?;
        self.consume(&TokenKind::Colon, "Expected ':' after function definition")?;
        let body = self.block()?;
        Ok(Statement::FuncDef { name, params, return_type, body, line, column })
    }

    fn try_statement(&mut self, line: usize, column: usize) -> Result<Statement, LucaError> {
        self.consume(&TokenKind::To, "Expected 'to' after 'try'")?;
        self.consume(&TokenKind::Colon, "Expected ':' after 'try to'")?;
        let try_block = self.block()?;
        let mut capture_block: Option<Block> = None;
        let mut finally_block: Option<Block> = None;
        if self.take(&TokenKind::Capture) {
            self.consume(&TokenKind::Error, "Expected 'error' after 'capture'")?;
            self.consume(&TokenKind::Colon, "Expected ':' after 'capture error'")?;
            capture_block = Some(self.block()?);
        }
        if self.take(&TokenKind::Finally) {
            self.consume(&TokenKind::Colon, "Expected ':' after 'finally'")?;
            finally_block = Some(self.block()?);
        }
        Ok(Statement::Try { try_block, capture_block, finally_block, line, column })
    }

    fn if_statement(&mut self, line: usize, column: usize) -> Result<Statement, LucaError> {
        // Already consumed 'if', now parse condition, then 'then', ':', block
        let condition = self.expression()?;
        self.consume(&TokenKind::Then, "Expected 'then' after if condition")?;
        self.consume(&TokenKind::Colon, "Expected ':' after 'then'")?;
        let first_block = self.block()?;
        let mut branches = vec![(condition, first_block)];
        let mut else_branch: Option<Block> = None;

        while self.take(&TokenKind::Else) {
            if self.take(&TokenKind::If) {
                // else if branch
                let cond = self.expression()?;
                self.consume(&TokenKind::Then, "Expected 'then' after 'else if' condition")?;
                self.consume(&TokenKind::Colon, "Expected ':' after 'then'")?;
                let block = self.block()?;
                branches.push((cond, block));
            } else {
                // else branch
                self.consume(&TokenKind::Colon, "Expected ':' after 'else'")?;
                let block = self.block()?;
                else_branch = Some(block);
                break;
            }
        }

        Ok(Statement::If { branches, else_branch, line, column })
    }

    fn for_statement(&mut self, line: usize, column: usize) -> Result<Statement, LucaError> {
        // Already consumed 'for', now check for `item from collection` vs `N times`
        // Look ahead: if next is Identifier and the token after that is `from`, then it's the collection form
        let is_for_each = if let TokenKind::Identifier(_) = &self.peek().kind {
            if self.current + 1 < self.tokens.len() {
                self.tokens[self.current + 1].kind == TokenKind::From
            } else {
                false
            }
        } else {
            false
        };
        if is_for_each {
            let (item, _, _) = self.identifier("Expected loop variable after 'for'")?;
            self.consume(&TokenKind::From, "Expected 'from' after loop variable")?;
            let collection = self.expression()?;
            self.consume(&TokenKind::Repeat, "Expected 'repeat' after collection")?;
            self.consume(&TokenKind::Colon, "Expected ':' after 'repeat'")?;
            let body = self.block()?;
            return Ok(Statement::ForEach { item, collection, body, line, column });
        }
        // Otherwise, it's `for N times repeat :`
        let count = self.expression()?;
        self.consume(&TokenKind::Times, "Expected 'times' after count")?;
        self.consume(&TokenKind::Repeat, "Expected 'repeat' after 'times'")?;
        self.consume(&TokenKind::Colon, "Expected ':' after 'repeat'")?;
        let body = self.block()?;
        Ok(Statement::ForTimes { count, body, line, column })
    }

    fn while_statement(&mut self, line: usize, column: usize) -> Result<Statement, LucaError> {
        let condition = self.expression()?;
        self.consume(&TokenKind::Repeat, "Expected 'repeat' after while condition")?;
        self.consume(&TokenKind::Colon, "Expected ':' after 'repeat'")?;
        let body = self.block()?;
        Ok(Statement::While { condition, body, line, column })
    }

    fn block(&mut self) -> Result<Block, LucaError> {
        // Expect newline, indent, statements, dedent
        // Colon already consumed, so next should be newline
        if !self.take(&TokenKind::Newline) {
            return Err(self.error("Expected newline after ':'"));
        }
        // Allow multiple newlines before indent (empty lines)
        while self.take(&TokenKind::Newline) {}
        self.consume(&TokenKind::Indent, "Expected indented block after ':'")?;
        let mut statements = Vec::new();
        while !self.check(&TokenKind::Dedent) && !self.check(&TokenKind::Eof) {
            if self.take(&TokenKind::Newline) {
                continue;
            }
            // Check for unexpected dedent with no statements? Empty block is error, handled after
            statements.push(self.statement()?);
            // After each statement, we expect newline or dedent
            // The program's outer loop handles newline, but inside block we need to handle
            // Consume optional newlines
            while self.take(&TokenKind::Newline) {}
        }
        if statements.is_empty() {
            return Err(self.error("Empty block is not allowed"));
        }
        self.consume(&TokenKind::Dedent, "Expected dedent after block")?;
        Ok(Block { statements })
    }

    fn expression(&mut self) -> Result<Expr, LucaError> { self.or() }

    fn or(&mut self) -> Result<Expr, LucaError> {
        let mut expr = self.and()?;
        while self.take(&TokenKind::Or) {
            let line = self.previous().line;
            let column = self.previous().column;
            let right = self.and()?;
            expr = Expr::Binary { left: Box::new(expr), op: BinaryOp::Or, right: Box::new(right), line, column };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LucaError> {
        let mut expr = self.comparison()?;
        while self.take(&TokenKind::And) {
            let line = self.previous().line;
            let column = self.previous().column;
            let right = self.comparison()?;
            expr = Expr::Binary { left: Box::new(expr), op: BinaryOp::And, right: Box::new(right), line, column };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LucaError> {
        let mut expr = self.term()?;
        while self.take(&TokenKind::Is) {
            let is_line = self.previous().line;
            let is_col = self.previous().column;
            let op = if self.take(&TokenKind::EqualEqual) {
                CompareOp::Equal
            } else if self.take(&TokenKind::NotEqual) {
                CompareOp::NotEqual
            } else if self.take(&TokenKind::GreaterEqual) {
                CompareOp::GreaterEqual
            } else if self.take(&TokenKind::LessEqual) {
                CompareOp::LessEqual
            } else if self.take(&TokenKind::Greater) {
                CompareOp::Greater
            } else if self.take(&TokenKind::Less) {
                CompareOp::Less
            } else {
                return Err(self.error("Expected comparison operator after 'is'"));
            };
            self.consume(&TokenKind::To, "Expected 'to' after comparison operator")?;
            let right = self.term()?;
            expr = Expr::Comparison { left: Box::new(expr), op, right: Box::new(right), line: is_line, column: is_col };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LucaError> {
        let mut expr = self.factor()?;
        loop {
            let op = if self.take(&TokenKind::Plus) {
                Some((BinaryOp::Add, self.previous().line, self.previous().column))
            } else if self.take(&TokenKind::Minus) {
                Some((BinaryOp::Subtract, self.previous().line, self.previous().column))
            } else {
                None
            };
            let Some((op, line, column)) = op else { break };
            let right = self.factor()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right), line, column };
        }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, LucaError> {
        let mut expr = self.unary()?;
        loop {
            let op = if self.take(&TokenKind::Star) {
                Some((BinaryOp::Multiply, self.previous().line, self.previous().column))
            } else if self.take(&TokenKind::Slash) {
                Some((BinaryOp::Divide, self.previous().line, self.previous().column))
            } else if self.take(&TokenKind::Percent) {
                Some((BinaryOp::Modulo, self.previous().line, self.previous().column))
            } else {
                None
            };
            let Some((op, line, column)) = op else { break };
            let right = self.unary()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right), line, column };
        }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<Expr, LucaError> {
        if self.take(&TokenKind::Not) {
            let token = self.previous().clone();
            return Ok(Expr::Unary { op: UnaryOp::Not, right: Box::new(self.unary()?), line: token.line, column: token.column });
        }
        if self.take(&TokenKind::Minus) {
            let token = self.previous().clone();
            return Ok(Expr::Unary { op: UnaryOp::Negate, right: Box::new(self.unary()?), line: token.line, column: token.column });
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, LucaError> {
        let mut expr = self.primary_single()?;
        loop {
            if self.check(&TokenKind::LeftBracket) {
                let bracket = self.peek().clone();
                self.consume(&TokenKind::LeftBracket, "Expected '['")?;
                let index = self.expression()?;
                self.consume(&TokenKind::RightBracket, "Expected ']' after index")?;
                expr = Expr::Index { base: Box::new(expr), index: Box::new(index), line: bracket.line, column: bracket.column };
            } else if self.check(&TokenKind::Dot) {
                let dot = self.peek().clone();
                self.consume(&TokenKind::Dot, "Expected '.'")?;
                let (method, _, _) = self.identifier("Expected method name after '.'")?;
                if method == "contains" {
                    self.consume(&TokenKind::LeftParen, "Expected '(' after 'contains'")?;
                    // Check for key = ... or value = ... forms (dict)
                    let mut mode = ContainsMode::Single;
                    let mut value: Option<Expr> = None;
                    if let TokenKind::Identifier(word) = self.peek().kind.clone() {
                        if (word == "key" || word == "value") && self.tokens.get(self.current + 1).is_some_and(|t| t.kind == TokenKind::Equal) {
                            self.advance(); // consume key/value
                            self.consume(&TokenKind::Equal, "Expected '='")?;
                            let v = self.expression()?;
                            mode = if word == "key" { ContainsMode::Key } else { ContainsMode::Value };
                            value = Some(v);
                        }
                    }
                    let value = match value {
                        Some(v) => v,
                        None => self.expression()?,
                    };
                    self.consume(&TokenKind::RightParen, "Expected ')' after argument")?;
                    expr = Expr::Contains { object: Box::new(expr), mode, value: Box::new(value), line: dot.line, column: dot.column };
                } else if method == "length" {
                    // length, length(), length(decimals), length(items)
                    if self.take(&TokenKind::LeftParen) {
                        if self.take(&TokenKind::RightParen) {
                            expr = Expr::Length { object: Box::new(expr), arg: None, line: dot.line, column: dot.column };
                        } else {
                            let (arg_name, _, _) = self.identifier("Expected 'decimals' or 'items'")?;
                            let arg = match arg_name.as_str() {
                                "decimals" => LengthArg::Decimals,
                                "items" => LengthArg::Items,
                                _ => return Err(self.error("Expected 'decimals' or 'items' in length()")),
                            };
                            self.consume(&TokenKind::RightParen, "Expected ')' after length argument")?;
                            expr = Expr::Length { object: Box::new(expr), arg: Some(arg), line: dot.line, column: dot.column };
                        }
                    } else {
                        expr = Expr::Length { object: Box::new(expr), arg: None, line: dot.line, column: dot.column };
                    }
                } else if method == "add" || method == "remove" || method == "clear" {
                    return Err(LucaError::new(format!("'{method}' is a statement, not an expression"), dot.line, dot.column));
                } else if self.take(&TokenKind::LeftParen) {
                    // Generic member call: Module.func(args) — resolved at runtime
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            args.push(self.expression()?);
                            if self.take(&TokenKind::Comma) {
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                    expr = Expr::MemberCall { base: Box::new(expr), member: method, args, line: dot.line, column: dot.column };
                } else {
                    expr = Expr::Member { base: Box::new(expr), member: method, line: dot.line, column: dot.column };
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary_single(&mut self) -> Result<Expr, LucaError> {
        let token = self.advance().clone();
        Ok(match token.kind {
            TokenKind::Int(value) => Expr::Literal(Value::Int(value)),
            TokenKind::Dec(value) => Expr::Literal(Value::Dec(value)),
            TokenKind::String(value) => Expr::StringTemplate { value, line: token.line, column: token.column },
            TokenKind::True => Expr::Literal(Value::Bool(true)),
            TokenKind::False => Expr::Literal(Value::Bool(false)),
            TokenKind::None => Expr::Literal(Value::None),
            TokenKind::Ask => {
                let prompt = self.primary_single()?;
                Expr::Ask { prompt: Box::new(prompt), line: token.line, column: token.column }
            }
            TokenKind::Identifier(name) => {
                // Check if this is a call: name '(' args ')' vs variable with type assertion name '(' type ')'
                if self.check(&TokenKind::LeftParen) {
                    // Peek ahead to distinguish call vs type assertion
                    // Type assertion is '(' + type_name + ')' with no comma and single type
                    // Look ahead without consuming
                    let mut is_type_assertion = false;
                    if self.current + 2 < self.tokens.len() {
                        if let TokenKind::Identifier(type_name) = &self.tokens[self.current + 1].kind {
                            if Type::parse(type_name).is_some() && self.tokens[self.current + 2].kind == TokenKind::RightParen {
                                // Check what follows after ')': if next is ',' then it's likely call with type as argument, but type names are not valid variable names for args
                                // For now, treat single type in parens as type assertion, not call
                                is_type_assertion = true;
                            }
                        }
                    }
                    // Also handle empty '()' as call (no type assertion, since type assertion requires a type)
                    if self.tokens.get(self.current + 1).is_some_and(|t| t.kind == TokenKind::RightParen) {
                        is_type_assertion = false;
                    }
                    if is_type_assertion {
                        Expr::Variable { name, asserted_type: self.optional_type_assertion()?, line: token.line, column: token.column }
                    } else {
                        // Parse call: consume '(' then args then ')'
                        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
                        let mut args = Vec::new();
                        if !self.check(&TokenKind::RightParen) {
                            loop {
                                args.push(self.expression()?);
                                if self.take(&TokenKind::Comma) {
                                    continue;
                                } else {
                                    break;
                                }
                            }
                        }
                        self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                        Expr::Call { name, args, line: token.line, column: token.column }
                    }
                } else {
                    Expr::Variable { name, asserted_type: None, line: token.line, column: token.column }
                }
            }
            // `error` is a keyword, but the name bound by `capture error` must
            // stay readable (indexing into it works through `postfix`).
            TokenKind::Error => Expr::Variable { name: "error".to_owned(), asserted_type: None, line: token.line, column: token.column },
            TokenKind::LeftBracket => {
                // List literal: [items]
                let line = token.line;
                let column = token.column;
                if self.take(&TokenKind::RightBracket) {
                    Expr::List { items: Vec::new(), line, column }
                } else {
                    let mut items = Vec::new();
                    loop {
                        items.push(self.expression()?);
                        if self.take(&TokenKind::Comma) {
                            if self.check(&TokenKind::RightBracket) {
                                break;
                            }
                            continue;
                        } else {
                            break;
                        }
                    }
                    self.consume(&TokenKind::RightBracket, "Expected ']' after list")?;
                    Expr::List { items, line, column }
                }
            }
            TokenKind::LeftParen => {
                // Tuple, group, or empty tuple: (), (expr), (a, b, ...)
                let line = token.line;
                let column = token.column;
                if self.take(&TokenKind::RightParen) {
                    Expr::Tuple { items: Vec::new(), line, column }
                } else {
                    let first = self.expression()?;
                    if self.take(&TokenKind::Comma) {
                        let mut items = vec![first];
                        // Allow trailing comma: (a,) or (a, b,)
                        if !self.check(&TokenKind::RightParen) {
                            loop {
                                items.push(self.expression()?);
                                if self.take(&TokenKind::Comma) {
                                    if self.check(&TokenKind::RightParen) {
                                        break;
                                    }
                                    continue;
                                } else {
                                    break;
                                }
                            }
                        }
                        self.consume(&TokenKind::RightParen, "Expected ')' after tuple")?;
                        Expr::Tuple { items, line, column }
                    } else {
                        self.consume(&TokenKind::RightParen, "Expected ')' after expression")?;
                        Expr::Group(Box::new(first))
                    }
                }
            }
            TokenKind::LeftBrace => {
                // Set or Dict literal, or empty set {}
                let line = token.line;
                let column = token.column;
                if self.take(&TokenKind::RightBrace) {
                    Expr::Set { items: Vec::new(), line, column }
                } else {
                    // Need to determine set vs dict: parse first expr, check for '='
                    // For dict, keys must be strings. Peek: if next is String and token after is '=', it's dict.
                    // Simpler: save position, try to parse as dict entry?
                    // Approach: if peek is String and token after string's closing? Hard without parsing.
                    // Instead: parse first expr, then check for '='.
                    let first = self.expression()?;
                    if self.take(&TokenKind::Equal) {
                        // Dict: first must be string
                        let key = match first {
                            Expr::Literal(Value::Str(s)) => s,
                            Expr::StringTemplate { value: s, .. } => {
                                // StringTemplate may contain interpolation; evaluate at runtime?
                                // For keys, require plain string without interpolation? Allow interpolation (evaluate later)?
                                // To keep keys static, require no '{' in template.
                                if s.contains('{') || s.contains('}') {
                                    return Err(LucaError::new("Dictionary keys must be plain strings", line, column));
                                }
                                s
                            }
                            _ => return Err(LucaError::new("Dictionary keys must be strings", line, column)),
                        };
                        // Parse value + optional (type)
                        let value = self.expression()?;
                        let value_type = self.optional_type_assertion()?;
                        let mut entries = vec![DictEntryExpr { key, value, value_type, line, column }];
                        while self.take(&TokenKind::Comma) {
                            if self.check(&TokenKind::RightBrace) {
                                break;
                            }
                            // Next entry: key must be string literal
                            let key_token = self.advance().clone();
                            let key = match key_token.kind {
                                TokenKind::String(s) => s,
                                _ => return Err(LucaError::new("Dictionary keys must be strings", key_token.line, key_token.column)),
                            };
                            self.consume(&TokenKind::Equal, "Expected '=' after dictionary key")?;
                            let value = self.expression()?;
                            let value_type = self.optional_type_assertion()?;
                            entries.push(DictEntryExpr { key, value, value_type, line: key_token.line, column: key_token.column });
                        }
                        self.consume(&TokenKind::RightBrace, "Expected '}' after dictionary")?;
                        Expr::Dict { entries, line, column }
                    } else {
                        // Set: first was element, parse rest
                        let mut items = vec![first];
                        while self.take(&TokenKind::Comma) {
                            if self.check(&TokenKind::RightBrace) {
                                break;
                            }
                            items.push(self.expression()?);
                        }
                        self.consume(&TokenKind::RightBrace, "Expected '}' after set")?;
                        Expr::Set { items, line, column }
                    }
                }
            }
            TokenKind::Raise => return Err(LucaError::new("'raise' is a statement, not an expression", token.line, token.column)),
            _ => return Err(LucaError::new("Expected an expression", token.line, token.column)),
        })
    }
    fn optional_type_assertion(&mut self) -> Result<Option<Type>, LucaError> {
        if !self.take(&TokenKind::LeftParen) { return Ok(None); }
        let (name, _, _) = self.identifier("Expected a type name")?;
        let ty = Type::parse(&name).ok_or_else(|| self.error("Unknown type; expected int, dec, str, bool, or uni"))?;
        self.consume(&TokenKind::RightParen, "Expected ')' after type name")?;
        Ok(Some(ty))
    }
    fn identifier(&mut self, message: &str) -> Result<(String, usize, usize), LucaError> {
        let token = self.advance().clone();
        if let TokenKind::Identifier(name) = token.kind { Ok((name, token.line, token.column)) } else { Err(LucaError::new(message, token.line, token.column)) }
    }
    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<(), LucaError> { if self.take(kind) { Ok(()) } else { Err(self.error(message)) } }
    fn take(&mut self, kind: &TokenKind) -> bool { if self.check(kind) { self.advance(); true } else { false } }
    fn check(&self, kind: &TokenKind) -> bool { std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind) }
    fn advance(&mut self) -> &Token { if !self.check(&TokenKind::Eof) { self.current += 1; } self.previous() }
    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn previous(&self) -> &Token { &self.tokens[self.current.saturating_sub(1)] }
    fn error(&self, message: &str) -> LucaError { LucaError::new(message, self.peek().line, self.peek().column) }
}
