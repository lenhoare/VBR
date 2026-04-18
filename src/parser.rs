use crate::ast::*;
use crate::lexer::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) -> &Token {
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
        self.current()
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if *self.current() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, self.current()),
                line: 0,
            })
        }
    }

    fn expect_semicolon(&mut self) -> Result<(), ParseError> {
        match self.current() {
            Token::Semicolon => { self.advance(); Ok(()) }
            _ => Ok(()),
        }
    }

    fn parse_prim_type(&mut self) -> Result<Type, ParseError> {
        let typ = match self.current() {
            Token::Integer => Type::I32,
            Token::Long => Type::I32,
            Token::LongLong => Type::I64,
            Token::Single => Type::F32,
            Token::Double => Type::F64,
            Token::Boolean => Type::Bool,
            Token::Byte => Type::U8,
            Token::StringType => Type::String,
            Token::HashMap => {
                self.advance();
                self.expect(Token::LParen)?;
                let key = Box::new(self.parse_prim_type()?);
                self.expect(Token::Comma)?;
                let value = Box::new(self.parse_prim_type()?);
                self.expect(Token::RParen)?;
                Type::HashMap(key, value)
            }
            Token::Vec => {
                self.advance();
                self.expect(Token::LParen)?;
                let inner = Box::new(self.parse_prim_type()?);
                self.expect(Token::RParen)?;
                Type::Vec(inner)
            }
            Token::Ident(ref name) if name == "Result" => {
                self.advance();
                self.expect(Token::LParen)?;
                let ok_type = Box::new(self.parse_prim_type()?);
                self.expect(Token::Comma)?;
                let err_type = Box::new(self.parse_prim_type()?);
                self.expect(Token::RParen)?;
                Type::Result(ok_type, err_type)
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                self.advance();
                Type::UserDefined(name)
            }
            _ => return Err(ParseError {
                message: format!("Expected type, found {:?}", self.current()),
                line: 0,
            }),
        };
        self.advance();
        Ok(typ)
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();
        while self.current() != &Token::EOF {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current() {
            Token::Dim => self.parse_dim(),
            Token::Const => self.parse_const(),
            Token::Set => self.parse_set(),
            Token::If => self.parse_if(),
            Token::Select => self.parse_select(),
            Token::For => self.parse_for(),
            // Token::ForEach => self.parse_for_each(),
            Token::While => self.parse_while(),
            Token::Do => self.parse_do_while(),
            Token::Function => self.parse_function(),
            Token::Return => self.parse_return(),
            Token::Match => self.parse_match(),
            Token::Continue => {
                self.advance();
                Ok(Statement::Continue)
            }
            Token::Exit => {
                self.advance();
                Ok(Statement::ExitLoop)
            }
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_dim(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Dim
        let mut mutable = false;
        if self.current() == &Token::Mut {
            mutable = true;
            self.advance();
        }

        let name = match self.current() {
            Token::Ident(n) => { let n = n.clone(); self.advance(); n },
            _ => return Err(ParseError { message: format!("Expected identifier, found {:?}", self.current()), line: 0 }),
        };

        self.expect(Token::Colon)?;
        let typ = self.parse_prim_type()?;

        let init = if self.current() == &Token::Equal {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else { None };

        self.expect_semicolon()?;
        Ok(Statement::Dim { mutable, typ, name, init })
    }

    fn parse_const(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Const
        let mut pub_ = false;
        if self.current() == &Token::Public {
            pub_ = true;
            self.advance();
        }

        let name = match self.current() {
            Token::Ident(n) => { let n = n.clone(); self.advance(); n },
            _ => return Err(ParseError { message: format!("Expected identifier, found {:?}", self.current()), line: 0 }),
        };

        self.expect(Token::Equal)?;
        let value = self.parse_expr()?;
        self.expect_semicolon()?;
        Ok(Statement::Const { pub_, name, value })
    }

    fn parse_set(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Set
        let mut mutable = false;
        if self.current() == &Token::Mut {
            mutable = true;
            self.advance();
        }

        let target = self.parse_primary()?;
        self.expect(Token::Equal)?;
        let value = Box::new(self.parse_expr()?);
        self.expect_semicolon()?;
        Ok(Statement::Set { mutable, target, value })
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // If
        self.expect(Token::LParen)?;
        let condition = Box::new(self.parse_expr()?);
        self.expect(Token::RParen)?;
        let then_block = self.parse_block()?;
        let mut else_block = Vec::new();
        if self.current() == &Token::Else {
            self.advance();
            else_block = self.parse_block()?;
        }
        Ok(Statement::If { condition, then_block, else_block })
    }

    fn parse_select(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Select

        let value = if self.current() == &Token::LParen {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect(Token::RParen)?;
            expr
        } else {
            self.parse_expr()?
        };

        let mut arms = Vec::new();
        while self.current() == &Token::Case {
            self.advance();
            let is_else = self.current() == &Token::CaseElse;
            if is_else {
                self.advance();
            }

            let body = self.parse_block()?;
            
            if is_else {
                arms.push(SelectArm::Value { value: self.parse_expr()?, body });
            } else {
                self.expect(Token::Then)?;
                arms.push(SelectArm::Value { value: self.parse_expr()?, body });
            }
        }
        self.expect(Token::End)?;
        Ok(Statement::Select { value: Box::new(value), arms, else_arm: None })
    }

    fn parse_match(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Match
        let value = if self.current() == &Token::LParen {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect(Token::RParen)?;
            expr
        } else {
            self.parse_expr()?
        };

        let mut arms = Vec::new();
        while self.current() != &Token::End {
            self.advance();
            let pattern = self.parse_expr()?;
            self.expect(Token::Then)?;
            let body = self.parse_block()?;
            arms.push(MatchArm { pattern, body });
        }
        self.expect(Token::End)?;
        Ok(Statement::Match { value: Box::new(value), arms })
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // For
        let variable = match self.current() {
            Token::Ident(n) => { let n = n.clone(); self.advance(); n },
            _ => return Err(ParseError { message: format!("Expected identifier, found {:?}", self.current()), line: 0 }),
        };

        self.expect(Token::Equal)?;
        let start = Box::new(self.parse_expr()?);
        self.expect(Token::To)?;
        let end = Box::new(self.parse_expr()?);
        let step = if self.current() == &Token::Step {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else { None };

        let body = self.parse_block()?;
        Ok(Statement::For { variable, start, end, step, body })
    }

    fn parse_for_each(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // ForEach
        let variable = match self.current() {
            Token::Ident(n) => { let n = n.clone(); self.advance(); n },
            _ => return Err(ParseError { message: format!("Expected identifier, found {:?}", self.current()), line: 0 }),
        };
        self.expect(Token::In)?;
        let collection = Box::new(self.parse_expr()?);
        let body = self.parse_block()?;
        Ok(Statement::ForEach { variable, collection, body })
    }

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // While
        let condition = Box::new(self.parse_expr()?);
        let body = self.parse_block()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_do_while(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Do
        let body = self.parse_block()?;
        self.expect(Token::Loop)?;
        self.expect(Token::While)?;
        let condition = Box::new(self.parse_expr()?);
        self.expect_semicolon()?;
        Ok(Statement::DoWhile { condition, body, until: false })
    }

    fn parse_function(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Function
        let name = match self.current() {
            Token::Ident(n) => { let n = n.clone(); self.advance(); n },
            _ => return Err(ParseError { message: format!("Expected identifier, found {:?}", self.current()), line: 0 }),
        };

        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while self.current() != &Token::RParen {
            params.push(self.parse_param()?);
            if self.current() == &Token::Comma {
                self.advance();
            } else { break; }
        }
        self.expect(Token::RParen)?;

        let return_type = if self.current() == &Token::As {
            self.advance();
            Some(self.parse_prim_type()?)
        } else { None };

        let body = self.parse_block()?;
        Ok(Statement::Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let mut by_ref = false;
        let mut mut_ = false;
        if self.current() == &Token::ByRef {
            by_ref = true;
            self.advance();
        } else if self.current() == &Token::ByVal {
            self.advance();
        }
        if self.current() == &Token::Mut {
            mut_ = true;
            self.advance();
        }

        let name = match self.current() {
            Token::Ident(n) => { let n = n.clone(); self.advance(); n },
            _ => return Err(ParseError { message: format!("Expected identifier, found {:?}", self.current()), line: 0 }),
        };

        self.expect(Token::Colon)?;
        let typ = self.parse_prim_type()?;
        Ok(Param {
            name,
            typ,
            by_ref,
            mut_,
        })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Return
        let expr = if self.current() != &Token::End && self.current() != &Token::Loop {
            Some(Box::new(self.parse_expr()?))
        } else { None };
        self.expect_semicolon()?;
        Ok(Statement::Return(expr))
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expr()?;
        self.expect_semicolon()?;
        Ok(Statement::Expr(Box::new(expr)))
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut stmts = Vec::new();
        while self.current() != &Token::End && self.current() != &Token::EOF {
            stmts.push(self.parse_statement()?);
        }
        if self.current() == &Token::End {
            self.advance();
        }
        Ok(stmts)
    }

    fn parse_expr(&mut self) -> Result<Expression, ParseError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive()?;
        while let Token::Equal | Token::NotEqual | Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual = self.current() {
            let op = match self.current() {
                Token::Equal => { self.advance(); BinaryOp::Eq },
                Token::NotEqual => { self.advance(); BinaryOp::Ne },
                Token::Less => { self.advance(); BinaryOp::Lt },
                Token::LessEqual => { self.advance(); BinaryOp::Le },
                Token::Greater => { self.advance(); BinaryOp::Gt },
                Token::GreaterEqual => { self.advance(); BinaryOp::Ge },
                _ => unreachable!(),
            };
            let right = Box::new(self.parse_additive()?);
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right,
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative()?;
        while let Token::Plus | Token::Minus = self.current() {
            let op = match self.current() {
                Token::Plus => { self.advance(); BinaryOp::Add },
                Token::Minus => { self.advance(); BinaryOp::Sub },
                _ => unreachable!(),
            };
            let right = Box::new(self.parse_multiplicative()?);
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right,
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_primary()?;
        while let Token::Star | Token::Slash | Token::Mod = self.current() {
            let op = match self.current() {
                Token::Star => { self.advance(); BinaryOp::Mul },
                Token::Slash => { self.advance(); BinaryOp::Div },
                Token::Mod => { self.advance(); BinaryOp::Mod },
                _ => unreachable!(),
            };
            let right = Box::new(self.parse_primary()?);
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right,
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match self.current() {
            Token::Integer => {
                if let Token::Integer = self.current() {
                    let n = 0;
                    self.advance();
                    Ok(Expression::Integer(n))
                } else { unreachable!() }
            }
            Token::StringLiteral(_) => {
                if let Token::StringLiteral(_) = self.current() {
                    let s = "".to_string();
                    self.advance();
                    Ok(Expression::StringLiteral(s))
                } else { unreachable!() }
            }
            Token::True => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            Token::Nothing => {
                self.advance();
                Ok(Expression::Nothing)
            }
            Token::Null => {
                self.advance();
                Ok(Expression::Null)
            }
            Token::Ident(_) => {
                if let Token::Ident(name) = &self.current() {
                    let name = name.clone();
                    self.advance();
                    if self.current() == &Token::LParen {
                        self.advance();
                        let mut args = Vec::new();
                        while self.current() != &Token::RParen {
                            args.push(self.parse_expr()?);
                            if self.current() == &Token::Comma {
                                self.advance();
                            }
                        }
                        self.expect(Token::RParen)?;
                        Ok(Expression::Call {
                            function: Box::new(Expression::Ident(name)),
                            args,
                        })
                    } else if self.current() == &Token::Dot {
                        self.advance();
                        let field = match self.current() {
                            Token::Ident(f) => { let f = f.clone(); self.advance(); f },
                            _ => return Err(ParseError { message: format!("Expected field name, found {:?}", self.current()), line: 0 }),
                        };
                        Ok(Expression::FieldAccess {
                            object: Box::new(Expression::Ident(name)),
                            field,
                        })
                    } else {
                        Ok(Expression::Ident(name))
                    }
                } else { unreachable!() }
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(ParseError {
                message: format!("Unexpected token in primary: {:?}", self.current()),
                line: 0,
            }),
        }
    }
}
