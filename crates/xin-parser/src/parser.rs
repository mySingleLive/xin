//! Parser implementation

use std::iter::Peekable;
use std::str::Chars;

use xin_ast::*;
use xin_diagnostics::{SourceLocation, SourceSpan};
use xin_lexer::Lexer;

use crate::ParserError;

/// Parser state
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(lexer: &mut Lexer) -> Result<Self, ParserError> {
        let tokens = lexer.tokenize().map_err(|e| {
            ParserError::LexerError(e.to_string())
        })?;
        Ok(Self { tokens, current: 0 })
    }

    pub fn parse(&mut self) -> Result<SourceFile, ParserError> {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            declarations.push(self.parse_declaration()?);
        }

        Ok(SourceFile { declarations })
    }

    // ==================== Declaration Parsing ====================

    fn parse_declaration(&mut self) -> Result<Decl, ParserError> {
        let is_public = self.match_kind(TokenKind::Pub);

        let decl = match self.peek().kind {
            TokenKind::Func => self.parse_func_decl(is_public),
            TokenKind::Struct => self.parse_struct_decl(is_public),
            TokenKind::Interface => self.parse_interface_decl(is_public),
            TokenKind::Import => self.parse_import_decl(),
            _ => {
                // Try to parse as expression statement
                if is_public {
                    let token = self.advance();
                    Err(ParserError::unexpected_token(
                        "declaration",
                        token.kind,
                    ))
                } else {
                    // Rewind and try to parse as expression statement
                    self.parse_expr_decl()
                }
            }
        }?;

        Ok(decl)
    }

    fn parse_expr_decl(&mut self) -> Result<Decl, ParserError> {
        // Parse as a statement wrapped in a dummy declaration
        let stmt = self.parse_stmt()?;
        let span = stmt.span.clone();
        // For MVP, we'll just wrap statements in a block
        // This allows top-level expressions
        Ok(Decl::new(
            DeclKind::Func(FuncDecl {
                name: "__top_level__".to_string(),
                params: vec![],
                return_type: None,
                body: FuncBody::Block(vec![stmt]),
                is_public: false,
            }),
            span,
        ))
    }

    fn parse_func_decl(&mut self, is_public: bool) -> Result<Decl, ParserError> {
        self.consume(TokenKind::Func, "expected 'func'")?;

        let name = self.consume_ident("expected function name")?;
        self.consume(TokenKind::LParen, "expected '('")?;

        let params = self.parse_params()?;

        self.consume(TokenKind::RParen, "expected ')'")?;

        // Return type: check if next token is a type (not { or ->)
        let return_type = if self.check_type_start() {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.match_kind(TokenKind::Arrow) {
            // Expression body
            FuncBody::Expr(self.parse_expr()?)
        } else {
            // Block body
            self.consume(TokenKind::LBrace, "expected '{'")?;
            let stmts = self.parse_block()?;
            FuncBody::Block(stmts)
        };

        let span = self.span_from(1, 1);

        Ok(Decl::new(
            DeclKind::Func(FuncDecl {
                name,
                params,
                return_type,
                body,
                is_public,
            }),
            span,
        ))
    }

    fn parse_params(&mut self) -> Result<Vec<FuncParam>, ParserError> {
        let mut params = Vec::new();

        if !self.check(TokenKind::RParen) {
            loop {
                let mutable = self.match_kind(TokenKind::Var);
                let name = self.consume_ident("expected parameter name")?;
                self.consume(TokenKind::Colon, "expected ':'")?;
                let type_annotation = self.parse_type()?;

                params.push(FuncParam {
                    name,
                    type_annotation,
                    mutable,
                });

                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_struct_decl(&mut self, is_public: bool) -> Result<Decl, ParserError> {
        self.consume(TokenKind::Struct, "expected 'struct'")?;

        let name = self.consume_ident("expected struct name")?;

        let implements = if self.match_kind(TokenKind::Implements) {
            Some(self.consume_ident("expected interface name")?)
        } else {
            None
        };

        self.consume(TokenKind::LBrace, "expected '{'")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            if self.check(TokenKind::Func) {
                if let DeclKind::Func(f) = self.parse_func_decl(false)?.kind {
                    methods.push(f);
                }
            } else {
                let is_pub = self.match_kind(TokenKind::Pub);
                let field_name = self.consume_ident("expected field name")?;
                self.consume(TokenKind::Colon, "expected ':'")?;
                let type_annotation = self.parse_type()?;

                fields.push(StructField {
                    name: field_name,
                    type_annotation,
                    is_public: is_pub,
                });
            }
        }

        self.consume(TokenKind::RBrace, "expected '}'")?;

        let span = self.span_from(1, 1);

        Ok(Decl::new(
            DeclKind::Struct(StructDecl {
                name,
                fields,
                methods,
                implements,
                is_public,
            }),
            span,
        ))
    }

    fn parse_interface_decl(&mut self, is_public: bool) -> Result<Decl, ParserError> {
        self.consume(TokenKind::Interface, "expected 'interface'")?;

        let name = self.consume_ident("expected interface name")?;

        self.consume(TokenKind::LBrace, "expected '{'")?;

        let mut methods = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let is_mutating = self.match_kind(TokenKind::Mut);
            self.consume(TokenKind::Func, "expected 'func'")?;

            let method_name = self.consume_ident("expected method name")?;
            self.consume(TokenKind::LParen, "expected '('")?;

            let params = self.parse_params()?;

            self.consume(TokenKind::RParen, "expected ')'")?;

            let return_type = if self.match_kind(TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            methods.push(InterfaceMethod {
                name: method_name,
                params,
                return_type,
                is_mutating,
            });
        }

        self.consume(TokenKind::RBrace, "expected '}'")?;

        let span = self.span_from(1, 1);

        Ok(Decl::new(
            DeclKind::Interface(InterfaceDecl {
                name,
                methods,
                is_public,
            }),
            span,
        ))
    }

    fn parse_import_decl(&mut self) -> Result<Decl, ParserError> {
        self.consume(TokenKind::Import, "expected 'import'")?;

        let module = self.consume_ident("expected module name")?;

        let items = if self.match_kind(TokenKind::LBrace) {
            let mut items = Vec::new();
            loop {
                let name = self.consume_ident("expected import name")?;
                let alias = if self.match_kind(TokenKind::As) {
                    Some(self.consume_ident("expected alias")?)
                } else {
                    None
                };
                items.push(ImportItem { name, alias });
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
            self.consume(TokenKind::RBrace, "expected '}'")?;
            Some(items)
        } else {
            None
        };

        let span = self.span_from(1, 1);

        Ok(Decl::new(
            DeclKind::Import(ImportDecl { module, items }),
            span,
        ))
    }

    // ==================== Statement Parsing ====================

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        match self.peek().kind {
            TokenKind::Let | TokenKind::Var | TokenKind::ColonColon => self.parse_var_decl(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::LBrace => self.parse_block_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);

        let mutable = if self.match_kind(TokenKind::Var) {
            true
        } else if self.match_kind(TokenKind::Let) {
            false
        } else if self.match_kind(TokenKind::ColonColon) {
            // := syntax
            let name = self.consume_ident("expected variable name")?;
            let (type_annotation, object_mutable) = if self.match_kind(TokenKind::Colon) {
                let has_mut = self.match_kind(TokenKind::Mut);
                (Some(self.parse_type()?), has_mut)
            } else {
                (None, false)
            };

            let value = if self.match_kind(TokenKind::Eq) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            return Ok(Stmt::new(
                StmtKind::VarDecl(VarDecl {
                    name,
                    mutable: false,
                    type_annotation,
                    value,
                    object_mutable,
                }),
                span,
            ));
        } else {
            false
        };

        let name = self.consume_ident("expected variable name")?;

        let (type_annotation, object_mutable) = if self.match_kind(TokenKind::Colon) {
            let has_mut = self.match_kind(TokenKind::Mut);
            (Some(self.parse_type()?), has_mut)
        } else {
            (None, false)
        };

        let value = if self.match_kind(TokenKind::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Stmt::new(
            StmtKind::VarDecl(VarDecl {
                name,
                mutable,
                type_annotation,
                value,
                object_mutable,
            }),
            span,
        ))
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::Return, "expected 'return'")?;

        let value = if !self.check(TokenKind::RBrace) && !self.is_at_end() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Stmt::new(StmtKind::Return(value), span))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::If, "expected 'if'")?;

        let (condition, then_block, else_block) = self.parse_if_body(span.clone())?;

        Ok(Stmt::new(
            StmtKind::If {
                condition,
                then_block,
                else_block,
            },
            span,
        ))
    }

    /// Parse the body of an if statement (condition, then block, optional else)
    /// Used by both parse_if_stmt and else if handling
    fn parse_if_body(&mut self, span: SourceSpan) -> Result<(Expr, Vec<Stmt>, Option<Vec<Stmt>>), ParserError> {
        let condition = self.parse_expr()?;
        self.consume(TokenKind::LBrace, "expected '{'")?;
        let then_block = self.parse_block()?;

        let else_block = if self.match_kind(TokenKind::Else) {
            if self.match_kind(TokenKind::If) {
                // else if - recursively parse the next if
                let inner_span = self.span_from(self.peek().line, self.peek().column);
                let (inner_cond, inner_then, inner_else) = self.parse_if_body(inner_span)?;
                Some(vec![Stmt::new(
                    StmtKind::If {
                        condition: inner_cond,
                        then_block: inner_then,
                        else_block: inner_else,
                    },
                    span,
                )])
            } else {
                self.consume(TokenKind::LBrace, "expected '{'")?;
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok((condition, then_block, else_block))
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::For, "expected 'for'")?;

        // Check for different for variants
        if self.match_kind(TokenKind::LBrace) {
            // Infinite loop: for { }
            let body = self.parse_block()?;
            return Ok(Stmt::new(StmtKind::For(ForLoop::Infinite { body }), span));
        }

        if self.match_kind(TokenKind::LParen) {
            // Could be: for (item in list), for (i < n), for (init; cond; update)
            if self.check(TokenKind::Ident) && self.peek_next().kind == TokenKind::In {
                // for (item in list)
                let var_name = self.consume_ident("expected variable")?;
                self.consume(TokenKind::In, "expected 'in'")?;
                let iterable = self.parse_expr()?;
                self.consume(TokenKind::RParen, "expected ')'")?;
                self.consume(TokenKind::LBrace, "expected '{'")?;
                let body = self.parse_block()?;
                return Ok(Stmt::new(
                    StmtKind::For(ForLoop::ForIn {
                        var_name,
                        iterable,
                        body,
                    }),
                    span,
                ));
            }

            // C-style for or while-style
            // First, try to determine the type of for loop by looking at the structure
            // If it starts with 'let', 'var', or ':=' -> C-style with init
            // Otherwise, parse expression and check if followed by ';' or ')'

            let is_init_stmt = matches!(
                self.peek().kind,
                TokenKind::Let | TokenKind::Var | TokenKind::ColonColon
            );

            if is_init_stmt {
                // C-style for loop with init
                let init = Some(Box::new(self.parse_stmt()?));

                self.consume(TokenKind::Semicolon, "expected ';' after for init")?;

                let condition = if !self.check(TokenKind::Semicolon) && !self.check(TokenKind::RParen) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };

                if self.match_kind(TokenKind::RParen) {
                    // for (init; condition) - C-style with no update
                    self.consume(TokenKind::LBrace, "expected '{'")?;
                    let body = self.parse_block()?;
                    return Ok(Stmt::new(
                        StmtKind::For(ForLoop::CStyle {
                            init,
                            condition,
                            update: None,
                            body,
                        }),
                        span,
                    ));
                }

                self.consume(TokenKind::Semicolon, "expected ';' after for condition")?;
                let update = if !self.check(TokenKind::RParen) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.consume(TokenKind::RParen, "expected ')'")?;
                self.consume(TokenKind::LBrace, "expected '{'")?;
                let body = self.parse_block()?;
                return Ok(Stmt::new(
                    StmtKind::For(ForLoop::CStyle {
                        init,
                        condition,
                        update,
                        body,
                    }),
                    span,
                ));
            } else {
                // Could be while-style for (condition) or C-style without init
                let first_expr = self.parse_expr()?;

                if self.match_kind(TokenKind::RParen) {
                    // while-style: for (condition) { }
                    self.consume(TokenKind::LBrace, "expected '{'")?;
                    let body = self.parse_block()?;
                    return Ok(Stmt::new(
                        StmtKind::For(ForLoop::While {
                            condition: first_expr,
                            body,
                        }),
                        span,
                    ));
                }

                // C-style without init: for (; condition; update) or for (; condition)
                self.consume(TokenKind::Semicolon, "expected ';' or ')'")?;

                let condition = if !self.check(TokenKind::Semicolon) && !self.check(TokenKind::RParen) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };

                if self.match_kind(TokenKind::RParen) {
                    self.consume(TokenKind::LBrace, "expected '{'")?;
                    let body = self.parse_block()?;
                    return Ok(Stmt::new(
                        StmtKind::For(ForLoop::CStyle {
                            init: None,
                            condition,
                            update: None,
                            body,
                        }),
                        span,
                    ));
                }

                self.consume(TokenKind::Semicolon, "expected ';'")?;
                let update = if !self.check(TokenKind::RParen) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.consume(TokenKind::RParen, "expected ')'")?;
                self.consume(TokenKind::LBrace, "expected '{'")?;
                let body = self.parse_block()?;
                return Ok(Stmt::new(
                    StmtKind::For(ForLoop::CStyle {
                        init: None,
                        condition,
                        update,
                        body,
                    }),
                    span,
                ));
            }
        }

        // for condition { } - while-style without parens
        let condition = self.parse_expr()?;
        self.consume(TokenKind::LBrace, "expected '{'")?;
        let body = self.parse_block()?;
        Ok(Stmt::new(
            StmtKind::For(ForLoop::While { condition, body }),
            span,
        ))
    }

    fn peek_next(&self) -> &Token {
        if self.current + 1 >= self.tokens.len() {
            self.tokens.last().unwrap()
        } else {
            &self.tokens[self.current + 1]
        }
    }

    fn parse_block_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::LBrace, "expected '{'")?;
        let stmts = self.parse_block()?;
        Ok(Stmt::new(StmtKind::Block(stmts), span))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut stmts = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        self.consume(TokenKind::RBrace, "expected '}'")?;
        Ok(stmts)
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        let expr = self.parse_expr()?;
        Ok(Stmt::new(StmtKind::Expr(expr), span))
    }

    // ==================== Expression Parsing ====================

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.parse_elvis()?;

        if self.match_kind(TokenKind::Eq) {
            let value = self.parse_assignment()?;
            let span = expr.span.clone();
            return Ok(Expr::new(
                ExprKind::Assignment {
                    target: Box::new(expr),
                    value: Box::new(value),
                },
                span,
            ));
        }

        // Compound assignments
        let op_kind = self.peek().kind;
        let op = match op_kind {
            TokenKind::PlusEq => Some(BinOp::Add),
            TokenKind::MinusEq => Some(BinOp::Sub),
            TokenKind::StarEq => Some(BinOp::Mul),
            TokenKind::SlashEq => Some(BinOp::Div),
            TokenKind::PercentEq => Some(BinOp::Mod),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let value = self.parse_assignment()?;
            let span = expr.span.clone();
            // Desugar: a += b -> a = a + b
            return Ok(Expr::new(
                ExprKind::Assignment {
                    target: Box::new(expr.clone()),
                    value: Box::new(Expr::new(
                        ExprKind::Binary {
                            op,
                            left: Box::new(expr),
                            right: Box::new(value),
                        },
                        span.clone(),
                    )),
                },
                span,
            ));
        }

        Ok(expr)
    }

    fn parse_elvis(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_or()?;

        // Elvis operator (??)
        while self.match_kind(TokenKind::QuestionQuestion) {
            let right = self.parse_or()?;
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Elvis {
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            );
        }

        // Ternary conditional (condition ? then : else)
        // This has lower precedence than comparison operators
        if self.match_kind(TokenKind::Question) {
            let then_expr = self.parse_expr()?;
            self.consume(TokenKind::Colon, "expected ':'")?;
            let else_expr = self.parse_elvis()?; // Right-associative
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Conditional {
                    condition: Box::new(expr),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_and()?;

        while self.match_kind(TokenKind::OrOr) {
            let right = self.parse_and()?;
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Binary {
                    op: BinOp::Or,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_equality()?;

        while self.match_kind(TokenKind::AndAnd) {
            let right = self.parse_equality()?;
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Binary {
                    op: BinOp::And,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::Ne => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_term()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_factor()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = expr.span.clone();
            expr = Expr::new(
                ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        let op = match self.peek().kind {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Not => Some(UnaryOp::Not),
            _ => None,
        };

        if let Some(op) = op {
            let span = self.span_from(self.peek().line, self.peek().column);
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::new(
                ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
                span,
            ));
        }

        // Move expression
        if self.match_kind(TokenKind::Move) {
            let span = self.span_from(self.peek().line, self.peek().column);
            let expr = self.parse_unary()?;
            return Ok(Expr::new(ExprKind::Move(Box::new(expr)), span));
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_kind(TokenKind::QuestionDot) {
                // Safe navigation: obj?.field
                let field = self.consume_ident("expected field name")?;
                let span = expr.span.clone();
                expr = Expr::new(
                    ExprKind::SafeAccess {
                        object: Box::new(expr),
                        field,
                    },
                    span,
                );
            } else if self.match_kind(TokenKind::Dot) {
                // Field access or method call
                let name = self.consume_ident("expected field or method name")?;

                if self.match_kind(TokenKind::LParen) {
                    // Method call
                    let args = self.parse_args()?;
                    self.consume(TokenKind::RParen, "expected ')'")?;
                    let span = expr.span.clone();
                    expr = Expr::new(
                        ExprKind::MethodCall {
                            object: Box::new(expr),
                            method: name,
                            args,
                        },
                        span,
                    );
                } else {
                    let span = expr.span.clone();
                    expr = Expr::new(
                        ExprKind::FieldAccess {
                            object: Box::new(expr),
                            field: name,
                        },
                        span,
                    );
                }
            } else if self.match_kind(TokenKind::LBracket) {
                // Index access
                let index = self.parse_expr()?;
                self.consume(TokenKind::RBracket, "expected ']'")?;
                let span = expr.span.clone();
                expr = Expr::new(
                    ExprKind::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    },
                    span,
                );
            } else if self.match_kind(TokenKind::BangBang) {
                // Force unwrap
                let span = expr.span.clone();
                expr = Expr::new(ExprKind::ForceUnwrap(Box::new(expr)), span);
            } else if self.match_kind(TokenKind::LParen) {
                // Function call
                let args = self.parse_args()?;
                self.consume(TokenKind::RParen, "expected ')'")?;
                let span = expr.span.clone();
                expr = Expr::new(
                    ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                    span,
                );
            } else {
                break;
            }
        }

        // Ternary conditional is now handled in parse_elvis

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let token = self.peek();

        let span = self.span_from(token.line, token.column);

        match token.kind {
            TokenKind::IntLiteral => {
                let value: i64 = token.text.parse().unwrap();
                self.advance();
                Ok(Expr::new(ExprKind::IntLiteral(value), span))
            }
            TokenKind::FloatLiteral => {
                let value: f64 = token.text.parse().unwrap();
                self.advance();
                Ok(Expr::new(ExprKind::FloatLiteral(value), span))
            }
            TokenKind::StringLiteral => {
                let value = token.text.clone();
                self.advance();
                Ok(Expr::new(ExprKind::StringLiteral(value), span))
            }
            TokenKind::TemplateString => {
                let text = self.advance().text.clone();
                self.parse_template_literal(&text, span)
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLiteral(true), span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLiteral(false), span))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::new(ExprKind::Null, span))
            }
            TokenKind::Ident => {
                let name = token.text.clone();
                self.advance();
                // Check for struct instantiation (only for PascalCase type names)
                if self.check(TokenKind::LBrace) && name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    self.advance(); // consume LBrace
                    let mutable = false; // TODO: handle mut keyword
                    let fields = self.parse_struct_fields()?;
                    self.consume(TokenKind::RBrace, "expected '}'")?;
                    return Ok(Expr::new(
                        ExprKind::StructInstance {
                            name,
                            fields,
                            mutable,
                        },
                        span,
                    ));
                }
                Ok(Expr::new(ExprKind::Ident(name), span))
            }
            TokenKind::LParen => {
                self.advance();
                // Could be grouping, tuple, or lambda
                if self.check(TokenKind::RParen) {
                    // Empty parens - lambda with no params
                    self.consume(TokenKind::RParen, "expected ')'")?;
                    if self.match_kind(TokenKind::Arrow) {
                        let body = self.parse_lambda_body()?;
                        return Ok(Expr::new(
                            ExprKind::Lambda {
                                params: vec![],
                                return_type: None,
                                body,
                            },
                            span,
                        ));
                    }
                    return Err(ParserError::ExpectedExpression);
                }

                // Check if this is a lambda
                if self.is_lambda_start() {
                    return self.parse_lambda();
                }

                // Regular expression grouping
                let expr = self.parse_expr()?;
                self.consume(TokenKind::RParen, "expected ')'")?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                // Could be block expression or map literal
                self.advance();
                if self.check(TokenKind::RBrace) {
                    // Empty map
                    self.advance();
                    return Ok(Expr::new(ExprKind::MapLiteral(vec![]), span));
                }

                // Check first element to determine if map or block
                if self.check(TokenKind::Ident) && self.peek_next().kind == TokenKind::Colon {
                    // Map literal
                    let entries = self.parse_map_entries()?;
                    self.consume(TokenKind::RBrace, "expected '}'")?;
                    return Ok(Expr::new(ExprKind::MapLiteral(entries), span));
                }

                // Block expression
                let stmts = self.parse_block()?;
                Ok(Expr::new(
                    ExprKind::Lambda {
                        params: vec![],
                        return_type: None,
                        body: LambdaBody::Block(stmts),
                    },
                    span,
                ))
            }
            TokenKind::LBracket => {
                // Array literal
                self.advance();
                let elements = self.parse_array_elements()?;
                self.consume(TokenKind::RBracket, "expected ']'")?;
                Ok(Expr::new(ExprKind::ArrayLiteral(elements), span))
            }
            TokenKind::If => {
                // If expression
                self.consume(TokenKind::If, "expected 'if'")?;
                let condition = self.parse_expr()?;
                self.consume(TokenKind::LBrace, "expected '{'")?;
                let then_branch = self.parse_block_expr()?;
                let else_branch = if self.match_kind(TokenKind::Else) {
                    if self.match_kind(TokenKind::If) {
                        Some(Box::new(self.parse_if_expr()?))
                    } else {
                        self.consume(TokenKind::LBrace, "expected '{'")?;
                        Some(Box::new(self.parse_block_expr()?))
                    }
                } else {
                    None
                };
                Ok(Expr::new(
                    ExprKind::If {
                        condition: Box::new(condition),
                        then_branch: Box::new(then_branch),
                        else_branch,
                    },
                    span,
                ))
            }
            _ => Err(ParserError::ExpectedExpression),
        }
    }

    fn parse_template_literal(&mut self, raw: &str, span: SourceSpan) -> Result<Expr, ParserError> {
        let mut parts = Vec::new();
        let mut chars = raw.chars().peekable();
        let mut text = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        match escaped {
                            '`' | '\\' | '{' | '}' => text.push(escaped),
                            'n' => text.push('\n'),
                            't' => text.push('\t'),
                            'r' => text.push('\r'),
                            _ => {
                                return Err(ParserError::InvalidEscape(escaped));
                            }
                        }
                    }
                }
                '{' => {
                    if !text.is_empty() {
                        parts.push(TemplatePart::Text(text.clone()));
                        text.clear();
                    }
                    let expr = self.parse_template_expr(&mut chars)?;
                    parts.push(TemplatePart::Expr(Box::new(expr)));
                }
                _ => text.push(ch),
            }
        }

        if !text.is_empty() {
            parts.push(TemplatePart::Text(text));
        }

        Ok(Expr::new(ExprKind::TemplateLiteral(parts), span))
    }

    fn parse_template_expr(&mut self, chars: &mut Peekable<Chars>) -> Result<Expr, ParserError> {
        let mut expr_chars = String::new();
        let mut brace_count = 1;
        let mut string_delim: Option<char> = None;

        while brace_count > 0 {
            let ch = chars.next().ok_or(ParserError::UnclosedTemplateExpr)?;

            match (ch, string_delim) {
                ('"' | '\'', None) => {
                    string_delim = Some(ch);
                    expr_chars.push(ch);
                }
                (c, Some(d)) if c == d => {
                    string_delim = None;
                    expr_chars.push(ch);
                }
                ('\\', Some(_)) => {
                    expr_chars.push(ch);
                    if let Some(next) = chars.next() {
                        expr_chars.push(next);
                    }
                }
                ('{', None) => {
                    brace_count += 1;
                    expr_chars.push(ch);
                }
                ('}', None) => {
                    brace_count -= 1;
                    if brace_count > 0 {
                        expr_chars.push(ch);
                    }
                }
                _ => expr_chars.push(ch),
            }
        }

        // Parse the collected expression string
        self.parse_expr_from_str(&expr_chars)
    }

    fn parse_expr_from_str(&mut self, expr_str: &str) -> Result<Expr, ParserError> {
        // Create a temporary lexer and parser for the expression
        let mut temp_lexer = Lexer::new(expr_str);
        let temp_tokens = temp_lexer.tokenize().map_err(|e| {
            ParserError::LexerError(e.to_string())
        })?;

        // Save current state
        let original_tokens = std::mem::replace(&mut self.tokens, temp_tokens);
        let original_current = self.current;
        self.current = 0;

        // Parse expression
        let result = self.parse_expr();

        // Restore state
        self.tokens = original_tokens;
        self.current = original_current;

        result
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::If, "expected 'if'")?;
        let condition = self.parse_expr()?;
        self.consume(TokenKind::LBrace, "expected '{'")?;
        let then_branch = self.parse_block_expr()?;
        let else_branch = if self.match_kind(TokenKind::Else) {
            if self.match_kind(TokenKind::If) {
                Some(Box::new(self.parse_if_expr()?))
            } else {
                self.consume(TokenKind::LBrace, "expected '{'")?;
                Some(Box::new(self.parse_block_expr()?))
            }
        } else {
            None
        };
        Ok(Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch,
            },
            span,
        ))
    }

    fn parse_block_expr(&mut self) -> Result<Expr, ParserError> {
        let mut stmts = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        self.consume(TokenKind::RBrace, "expected '}'")?;

        Ok(Expr::new(
            ExprKind::Lambda {
                params: vec![],
                return_type: None,
                body: LambdaBody::Block(stmts),
            },
            self.span_from(1, 1),
        ))
    }

    fn is_lambda_start(&mut self) -> bool {
        let saved = self.current;
        let result = self.try_parse_lambda_params();
        self.current = saved;
        result
    }

    fn try_parse_lambda_params(&mut self) -> bool {
        loop {
            if !self.check(TokenKind::Ident) {
                return false;
            }
            self.advance();

            if !self.check(TokenKind::Colon) {
                return false;
            }
            self.advance();

            if !self.check_type_start() {
                return false;
            }
            while self.check_type_start() {
                self.advance();
            }

            if self.match_kind(TokenKind::Comma) {
                continue;
            }
            break;
        }

        self.check(TokenKind::RParen) && {
            self.advance();
            self.check(TokenKind::Arrow)
        }
    }

    fn check_type_start(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Int
                | TokenKind::Float
                | TokenKind::Bool
                | TokenKind::String
                | TokenKind::Void
                | TokenKind::Ident
                | TokenKind::Star
        )
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::LParen, "expected '('")?;

        let params = self.parse_lambda_params()?;

        self.consume(TokenKind::RParen, "expected ')'")?;

        let return_type = if self.match_kind(TokenKind::Arrow) {
            if self.check_type_start() && !self.check(TokenKind::LBrace) {
                Some(self.parse_type()?)
            } else {
                None
            }
        } else {
            None
        };

        if !self.match_kind(TokenKind::Arrow) {
            return Err(ParserError::unexpected_token("->", self.peek().kind));
        }

        let body = self.parse_lambda_body()?;

        Ok(Expr::new(
            ExprKind::Lambda {
                params,
                return_type,
                body,
            },
            span,
        ))
    }

    fn parse_lambda_params(&mut self) -> Result<Vec<LambdaParam>, ParserError> {
        let mut params = Vec::new();

        if !self.check(TokenKind::RParen) {
            loop {
                let name = self.consume_ident("expected parameter name")?;
                let type_annotation = if self.match_kind(TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                params.push(LambdaParam {
                    name,
                    type_annotation,
                });

                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_lambda_body(&mut self) -> Result<LambdaBody, ParserError> {
        if self.match_kind(TokenKind::LBrace) {
            let stmts = self.parse_block()?;
            Ok(LambdaBody::Block(stmts))
        } else {
            let expr = self.parse_expr()?;
            Ok(LambdaBody::Expr(Box::new(expr)))
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();

        if !self.check(TokenKind::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(args)
    }

    fn parse_array_elements(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut elements = Vec::new();

        if !self.check(TokenKind::RBracket) {
            loop {
                elements.push(self.parse_expr()?);
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(elements)
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<(String, Expr)>, ParserError> {
        let mut fields = Vec::new();

        if !self.check(TokenKind::RBrace) {
            loop {
                let name = self.consume_ident("expected field name")?;
                self.consume(TokenKind::Colon, "expected ':'")?;
                let value = self.parse_expr()?;
                fields.push((name, value));
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(fields)
    }

    fn parse_map_entries(&mut self) -> Result<Vec<(Expr, Expr)>, ParserError> {
        let mut entries = Vec::new();

        if !self.check(TokenKind::RBrace) {
            loop {
                let key = self.parse_expr()?;
                self.consume(TokenKind::Colon, "expected ':'")?;
                let value = self.parse_expr()?;
                entries.push((key, value));
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(entries)
    }

    // ==================== Type Parsing ====================

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        let mut ty = self.parse_base_type()?;

        // Handle nullable suffix
        while self.match_kind(TokenKind::Question) {
            ty = Type::Nullable(Box::new(ty));
        }

        // Handle array suffix
        while self.match_kind(TokenKind::LBracket) {
            self.consume(TokenKind::RBracket, "expected ']'")?;
            ty = Type::Array(Box::new(ty));
        }

        Ok(ty)
    }

    fn parse_base_type(&mut self) -> Result<Type, ParserError> {
        match self.peek().kind {
            TokenKind::Int => {
                self.advance();
                Ok(Type::Int)
            }
            TokenKind::Float => {
                self.advance();
                Ok(Type::Float)
            }
            TokenKind::Bool => {
                self.advance();
                Ok(Type::Bool)
            }
            TokenKind::String => {
                self.advance();
                Ok(Type::String)
            }
            TokenKind::Void => {
                self.advance();
                Ok(Type::Void)
            }
            TokenKind::Star => {
                // Pointer type
                self.advance();
                let mutable = self.match_kind(TokenKind::Mut);
                let inner = self.parse_base_type()?;
                Ok(Type::Pointer {
                    inner: Box::new(inner),
                    mutable,
                })
            }
            TokenKind::Ident => {
                let name = self.consume_ident("expected type name")?;

                // Check for generic
                if self.match_kind(TokenKind::Lt) {
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.consume(TokenKind::Gt, "expected '>'")?;
                    Ok(Type::Generic { name, args })
                } else {
                    Ok(Type::Named(name))
                }
            }
            TokenKind::Func => {
                // Function type: func(A, B) R
                self.advance();
                self.consume(TokenKind::LParen, "expected '('")?;

                let mut params = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        params.push(self.parse_type()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RParen, "expected ')'")?;

                let return_type = if self.match_kind(TokenKind::Arrow) {
                    self.parse_type()?
                } else {
                    Type::Void
                };

                Ok(Type::Function {
                    params,
                    return_type: Box::new(return_type),
                })
            }
            _ => Err(ParserError::unexpected_token("type", self.peek().kind)),
        }
    }

    // ==================== Helpers ====================

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParserError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(ParserError::unexpected_token(message, self.peek().kind))
        }
    }

    fn consume_ident(&mut self, message: &str) -> Result<String, ParserError> {
        if self.check(TokenKind::Ident) {
            Ok(self.advance().text)
        } else {
            Err(ParserError::unexpected_token(message, self.peek().kind))
        }
    }

    fn span_from(&self, line: usize, column: usize) -> SourceSpan {
        let start = SourceLocation::new(line, column, 0);
        let end = SourceLocation::new(self.peek().line, self.peek().column, 0);
        SourceSpan::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xin_lexer::Lexer;

    #[test]
    fn test_parse_func() {
        let source = r#"
            func add(a: int, b: int) int {
                return a + b
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert_eq!(file.declarations.len(), 1);
    }

    #[test]
    fn test_parse_struct() {
        let source = r#"
            struct User {
                name: string
                age: int
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert_eq!(file.declarations.len(), 1);
    }

    #[test]
    fn test_parse_var_decl() {
        let source = r#"
            let x = 10
            var y = 20
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 2);
    }

    #[test]
    fn test_parse_if_stmt() {
        let source = r#"
            if a > b {
                print(a)
            } else {
                print(b)
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_for_loop() {
        let source = r#"
            for (item in list) {
                print(item)
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }
}