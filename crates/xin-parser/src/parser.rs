//! Parser implementation

use std::iter::Peekable;
use std::str::Chars;

use xin_ast::*;
use xin_diagnostics::{SourceLocation, SourceSpan};
use xin_lexer::Lexer;

use crate::ParserError;

/// Result of parsing function call arguments
/// Contains both regular args and optional trailing lambda
struct CallArgs {
    args: Vec<Expr>,
    trailing_lambda: Option<Expr>,
    /// Whether the trailing lambda body has consumed the closing RParen
    rparen_consumed: bool,
}

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
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
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
            // If variable is declared with 'var', make the object mutable by default
            (None, mutable)
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

    fn parse_break_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::Break, "expected 'break'")?;
        Ok(Stmt::new(StmtKind::Break, span))
    }

    fn parse_continue_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::Continue, "expected 'continue'")?;
        Ok(Stmt::new(StmtKind::Continue, span))
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
                // Field access, method call, or map access with string key
                // Support:
                //   m.field       - identifier field access
                //   m."key"       - string literal key (equivalent to m["key"])
                //   m.'key'       - char literal key (equivalent to m['key'])
                //   m.`template`  - template string key (equivalent to m[`template`])

                let next_token = self.peek().clone();
                match next_token.kind {
                    TokenKind::StringLiteral => {
                        // Map access with string key: m."name"
                        self.advance();
                        let key_text = next_token.text.clone();
                        let key_span = self.span_from(next_token.line, next_token.column);
                        let key = Expr::new(ExprKind::StringLiteral(key_text), key_span);
                        let span = expr.span.clone();
                        expr = Expr::new(
                            ExprKind::Index {
                                object: Box::new(expr),
                                index: Box::new(key),
                            },
                            span,
                        );
                    }
                    TokenKind::CharLiteral => {
                        // Map access with char key: m.'name'
                        self.advance();
                        let key_text = next_token.text.clone();
                        let key_span = self.span_from(next_token.line, next_token.column);
                        let key = Expr::new(ExprKind::StringLiteral(key_text), key_span);
                        let span = expr.span.clone();
                        expr = Expr::new(
                            ExprKind::Index {
                                object: Box::new(expr),
                                index: Box::new(key),
                            },
                            span,
                        );
                    }
                    TokenKind::TemplateString => {
                        // Map access with template string key: m.`prefix_{key}`
                        self.advance();
                        let key_text = next_token.text.clone();
                        let key_span = self.span_from(next_token.line, next_token.column);
                        let key = self.parse_template_literal(&key_text, key_span)?;
                        let span = expr.span.clone();
                        expr = Expr::new(
                            ExprKind::Index {
                                object: Box::new(expr),
                                index: Box::new(key),
                            },
                            span,
                        );
                    }
                    _ => {
                        // Regular field access or method call
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
                    }
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
                // Function call with potential trailing lambda
                let call_args = self.parse_call_args()?;

                // Only consume RParen if not already consumed by trailing lambda
                if !call_args.rparen_consumed {
                    self.consume(TokenKind::RParen, "expected ')'")?;
                }

                // Add trailing lambda to args if present
                let mut args = call_args.args;
                if let Some(lambda) = call_args.trailing_lambda {
                    args.push(lambda);
                }

                // Check for trailing lambda block without params: test(1, 2) { }
                let trailing_block = if self.check(TokenKind::LBrace) {
                    // Check if this is a map literal or trailing lambda
                    // Trailing lambda if the next token after { is not an identifier followed by :
                    let saved = self.current;
                    self.advance(); // consume LBrace
                    if self.check(TokenKind::Ident) && self.peek_next().kind == TokenKind::Colon {
                        // This is a map literal, restore position
                        self.current = saved;
                        None
                    } else {
                        // This is a trailing lambda block
                        let stmts = self.parse_block()?;
                        let span = self.span_from(1, 1);
                        Some(Expr::new(
                            ExprKind::Lambda {
                                params: vec![],
                                return_type: None,
                                body: LambdaBody::Block(stmts),
                            },
                            span,
                        ))
                    }
                } else {
                    None
                };

                if let Some(lambda) = trailing_block {
                    args.push(lambda);
                }

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
            // Single-quoted string literal (treated as String for char() function)
            TokenKind::CharLiteral => {
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
            // Handle 'char' keyword as function call (char() builtin function)
            TokenKind::Char => {
                self.advance();
                // Treat 'char' as identifier for function call
                Ok(Expr::new(ExprKind::Ident("char".to_string()), span))
            }
            // Handle type keywords as identifiers for type conversion functions
            // Signed integer types: int8(), int16(), int32(), int64(), int128()
            TokenKind::Int8 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("int8".to_string()), span))
            }
            TokenKind::Int16 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("int16".to_string()), span))
            }
            TokenKind::Int32 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("int32".to_string()), span))
            }
            TokenKind::Int64 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("int64".to_string()), span))
            }
            TokenKind::Int128 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("int128".to_string()), span))
            }
            // Unsigned integer types: uint8(), uint16(), uint32(), uint64(), uint128()
            TokenKind::UInt8 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("uint8".to_string()), span))
            }
            TokenKind::UInt16 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("uint16".to_string()), span))
            }
            TokenKind::UInt32 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("uint32".to_string()), span))
            }
            TokenKind::UInt64 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("uint64".to_string()), span))
            }
            TokenKind::UInt128 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("uint128".to_string()), span))
            }
            // Byte type
            TokenKind::Byte => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("byte".to_string()), span))
            }
            // Floating-point types: float8(), float16(), float32(), float64(), float128()
            TokenKind::Float8 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("float8".to_string()), span))
            }
            TokenKind::Float16 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("float16".to_string()), span))
            }
            TokenKind::Float32 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("float32".to_string()), span))
            }
            TokenKind::Float64 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("float64".to_string()), span))
            }
            TokenKind::Float128 => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("float128".to_string()), span))
            }
            // Bool type: bool()
            TokenKind::Bool => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("bool".to_string()), span))
            }
            // String type: string()
            TokenKind::String => {
                self.advance();
                Ok(Expr::new(ExprKind::Ident("string".to_string()), span))
            }
            TokenKind::LParen => {
                // Could be grouping, tuple, or lambda
                // Don't consume LParen yet - let parse_lambda handle it if needed
                if self.check(TokenKind::RParen) {
                    // Empty parens - could be () for grouping or lambda
                    self.advance(); // consume LParen
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

                // Check if this is a lambda (peek ahead without consuming LParen)
                if self.is_lambda_start() {
                    return self.parse_lambda();
                }

                // Regular expression grouping
                self.advance(); // consume LParen
                let expr = self.parse_expr()?;
                self.consume(TokenKind::RParen, "expected ')'")?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                // Map literal only - block expressions (lambdas) are handled in parse_postfix
                self.advance();
                if self.check(TokenKind::RBrace) {
                    // Empty map
                    self.advance();
                    return Ok(Expr::new(ExprKind::MapLiteral(vec![]), span));
                }

                // Check if this looks like a map literal: { key: value, ... }
                // Map literals start with an identifier, string literal, or expression followed by colon
                // For simple cases, check if next token after current is Colon
                // For complex cases (function calls), we save position and try to parse
                let current = self.peek();
                let is_simple_key = matches!(
                    current.kind,
                    TokenKind::StringLiteral
                    | TokenKind::CharLiteral
                    | TokenKind::TemplateString
                    | TokenKind::Ident
                    | TokenKind::IntLiteral
                    | TokenKind::FloatLiteral
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::Null
                );

                let is_map_like = if is_simple_key && self.peek_next().kind == TokenKind::Colon {
                    // For simple keys followed by colon, it's definitely a map
                    true
                } else {
                    // For complex expressions, try to parse and check if followed by colon
                    // Save current position
                    let saved = self.current;

                    // Try to parse an expression
                    let result = self.parse_expr();

                    // Check if next token is colon
                    let is_colon = self.check(TokenKind::Colon);

                    // Restore position
                    self.current = saved;

                    // If parsing succeeded and is followed by colon, it's a map
                    result.is_ok() && is_colon
                };

                if !is_map_like {
                    // Not a map literal - this is likely a statement block, return error
                    // to let the caller handle it
                    return Err(ParserError::ExpectedExpression);
                }

                // Must be a map literal
                let entries = self.parse_map_entries()?;
                self.consume(TokenKind::RBrace, "expected '}'")?;
                Ok(Expr::new(ExprKind::MapLiteral(entries), span))
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
        // We're at LParen, need to check if this is a lambda
        let saved = self.current;
        self.advance(); // skip LParen
        let result = self.try_parse_lambda_params();
        self.current = saved;
        result
    }

    fn try_parse_lambda_params(&mut self) -> bool {
        // We're now after LParen, check for params
        // Handle empty params case: ()
        if self.check(TokenKind::RParen) {
            self.advance();
            // After ), we expect either -> or a return type followed by ->
            return self.check(TokenKind::Arrow) || self.check_type_start();
        }

        // Support both typed params (a: int, b: int) and inferred params (a, b)
        loop {
            if !self.check(TokenKind::Ident) {
                return false;
            }
            self.advance();

            // Check for type annotation (optional)
            if self.check(TokenKind::Colon) {
                // Type annotation present: a: int
                self.advance();
                if !self.check_type_start() {
                    return false;
                }
                while self.check_type_start() {
                    self.advance();
                }
            }
            // No type annotation: just 'a' - that's fine for type inference

            if self.match_kind(TokenKind::Comma) {
                continue;
            }
            break;
        }

        self.check(TokenKind::RParen) && {
            self.advance();
            // After ), we expect either -> or a return type followed by ->
            self.check(TokenKind::Arrow) || self.check_type_start()
        }
    }

    fn check_type_start(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Int8
                | TokenKind::Int16
                | TokenKind::Int32
                | TokenKind::Int64
                | TokenKind::Int128
                | TokenKind::UInt8
                | TokenKind::UInt16
                | TokenKind::UInt32
                | TokenKind::UInt64
                | TokenKind::UInt128
                | TokenKind::Byte
                | TokenKind::Float8
                | TokenKind::Float16
                | TokenKind::Float32
                | TokenKind::Float64
                | TokenKind::Float128
                | TokenKind::Char
                | TokenKind::Bool
                | TokenKind::String
                | TokenKind::Void
                | TokenKind::Ident
                | TokenKind::Star
                | TokenKind::Func  // Support function type as return type or annotation
        )
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParserError> {
        let span = self.span_from(self.peek().line, self.peek().column);
        self.consume(TokenKind::LParen, "expected '('")?;

        let params = self.parse_lambda_params()?;

        self.consume(TokenKind::RParen, "expected ')'")?;

        // Parse return type if present: (params) ReturnType -> body
        // But only if there's a type followed by ->
        let return_type = if self.check_type_start() {
            // Peek ahead to see if this is a return type (type followed by ->)
            let saved = self.current;
            let _ = self.parse_type();
            let has_arrow = self.check(TokenKind::Arrow);
            self.current = saved;
            if has_arrow {
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

    fn parse_call_args(&mut self) -> Result<CallArgs, ParserError> {
        let mut args = Vec::new();
        let mut trailing_lambda = None;
        let mut rparen_consumed = false;

        if !self.check(TokenKind::RParen) {
            loop {
                // Parse argument
                let arg = self.parse_expr()?;

                // Check if this is a typed lambda param: x: int
                // This happens when we have comma-separated trailing lambda params
                if self.check(TokenKind::Colon) && matches!(arg.kind, ExprKind::Ident(_)) {
                    // This might be a trailing lambda param - look ahead
                    if self.is_typed_trailing_lambda_param() {
                        // Parse the type annotation
                        self.consume(TokenKind::Colon, "expected ':'")?;
                        let _type = self.parse_type()?;

                        // Collect all params including the one we just started
                        let mut params = vec![LambdaParam {
                            name: match arg.kind {
                                ExprKind::Ident(name) => name,
                                _ => unreachable!(),
                            },
                            type_annotation: Some(_type),
                        }];

                        // Parse remaining params
                        while self.match_kind(TokenKind::Comma) {
                            let name = self.consume_ident("expected parameter name")?;
                            let type_annotation = if self.match_kind(TokenKind::Colon) {
                                Some(self.parse_type()?)
                            } else {
                                None
                            };
                            params.push(LambdaParam { name, type_annotation });
                        }

                        let lambda = self.parse_trailing_lambda_body(params)?;
                        trailing_lambda = Some(lambda);
                        rparen_consumed = true; // parse_trailing_lambda_body consumes RParen
                        break;
                    }
                }

                args.push(arg);

                // Check for comma (more args) or semicolon (trailing lambda params)
                if self.match_kind(TokenKind::Comma) {
                    continue;
                }

                // Check for trailing lambda params after semicolon: (1, 2; x, y)
                if self.match_kind(TokenKind::Semicolon) {
                    // Parse trailing lambda params
                    let params = self.parse_trailing_lambda_params()?;
                    let lambda = self.parse_trailing_lambda_body(params)?;
                    trailing_lambda = Some(lambda);
                    rparen_consumed = true; // parse_trailing_lambda_body consumes RParen
                    break;
                }

                break;
            }
        }

        Ok(CallArgs { args, trailing_lambda, rparen_consumed })
    }

    /// Check if current position looks like a typed trailing lambda param
    fn is_typed_trailing_lambda_param(&self) -> bool {
        // We're at ':', check if followed by type and then ')' or ','
        // This is called after we've seen an identifier
        if !self.check(TokenKind::Colon) {
            return false;
        }
        // Peek ahead to see if this looks like a type annotation
        // We need to look at the token after ':'
        if self.current + 1 < self.tokens.len() {
            self.check_type_start_at(self.current + 1)
        } else {
            false
        }
    }

    /// Check if token at given position is a type start
    fn check_type_start_at(&self, pos: usize) -> bool {
        if pos >= self.tokens.len() {
            return false;
        }
        matches!(
            self.tokens[pos].kind,
            TokenKind::Int8
                | TokenKind::Int16
                | TokenKind::Int32
                | TokenKind::Int64
                | TokenKind::Int128
                | TokenKind::UInt8
                | TokenKind::UInt16
                | TokenKind::UInt32
                | TokenKind::UInt64
                | TokenKind::UInt128
                | TokenKind::Byte
                | TokenKind::Float8
                | TokenKind::Float16
                | TokenKind::Float32
                | TokenKind::Float64
                | TokenKind::Float128
                | TokenKind::Char
                | TokenKind::Bool
                | TokenKind::String
                | TokenKind::Void
                | TokenKind::Ident
                | TokenKind::Star
        )
    }

    /// Parse trailing lambda params (after semicolon)
    fn parse_trailing_lambda_params(&mut self) -> Result<Vec<LambdaParam>, ParserError> {
        let mut params = Vec::new();

        loop {
            let name = self.consume_ident("expected parameter name")?;
            let type_annotation = if self.match_kind(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };

            params.push(LambdaParam { name, type_annotation });

            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    /// Parse trailing lambda body (the block after params)
    fn parse_trailing_lambda_body(&mut self, params: Vec<LambdaParam>) -> Result<Expr, ParserError> {
        self.consume(TokenKind::RParen, "expected ')'")?;

        // The body must be a block
        self.consume(TokenKind::LBrace, "expected '{' for trailing lambda body")?;
        let stmts = self.parse_block()?;

        let span = self.span_from(1, 1);
        Ok(Expr::new(
            ExprKind::Lambda {
                params,
                return_type: None,
                body: LambdaBody::Block(stmts),
            },
            span,
        ))
    }

    /// Parse args for function call (without trailing lambda support)
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
            // Signed integer types
            TokenKind::Int8 => {
                self.advance();
                Ok(Type::Int8)
            }
            TokenKind::Int16 => {
                self.advance();
                Ok(Type::Int16)
            }
            TokenKind::Int32 => {
                self.advance();
                Ok(Type::Int32)
            }
            TokenKind::Int64 => {
                self.advance();
                Ok(Type::Int64)
            }
            TokenKind::Int128 => {
                self.advance();
                Ok(Type::Int128)
            }
            // Unsigned integer types
            TokenKind::UInt8 => {
                self.advance();
                Ok(Type::UInt8)
            }
            TokenKind::UInt16 => {
                self.advance();
                Ok(Type::UInt16)
            }
            TokenKind::UInt32 => {
                self.advance();
                Ok(Type::UInt32)
            }
            TokenKind::UInt64 => {
                self.advance();
                Ok(Type::UInt64)
            }
            TokenKind::UInt128 => {
                self.advance();
                Ok(Type::UInt128)
            }
            TokenKind::Byte => {
                self.advance();
                Ok(Type::UInt8) // byte is an alias for uint8
            }
            // Float types
            TokenKind::Float8 => {
                self.advance();
                Ok(Type::Float8)
            }
            TokenKind::Float16 => {
                self.advance();
                Ok(Type::Float16)
            }
            TokenKind::Float32 => {
                self.advance();
                Ok(Type::Float32)
            }
            TokenKind::Float64 => {
                self.advance();
                Ok(Type::Float64)
            }
            TokenKind::Float128 => {
                self.advance();
                Ok(Type::Float128)
            }
            // Other types
            TokenKind::Char => {
                self.advance();
                Ok(Type::Char)
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
                // Function type: func(A, B) R or func(A, B) -> R
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

                // Return type: either -> Type or just Type (without ->)
                let return_type = if self.match_kind(TokenKind::Arrow) {
                    self.parse_type()?
                } else if self.check_type_start() {
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

    // ==================== Lambda Tests ====================

    #[test]
    fn test_parse_lambda_basic() {
        // Basic lambda with expression body
        let source = r#"
            let add = (a, b) -> a + b
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_lambda_with_types() {
        // Lambda with type annotations
        let source = r#"
            let add = (a: int32, b: int32) -> a + b
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_lambda_with_return_type() {
        // Lambda with explicit return type
        let source = r#"
            let add = (a: int32, b: int32) int32 -> a + b
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_lambda_block_body() {
        // Lambda with block body
        let source = r#"
            let multiply = (a, b) -> {
                return a * b
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_lambda_no_params() {
        // Lambda with no parameters
        let source = r#"
            let getValue = () -> 42
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_lambda_as_arg() {
        // Lambda as function argument
        let source = r#"
            apply([1, 2, 3], (x) -> x * 2)
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    // ==================== Trailing Lambda Tests ====================

    #[test]
    fn test_parse_trailing_lambda_no_params() {
        // Trailing lambda without params: test { }
        let source = r#"
            forEach {
                print("hello")
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_trailing_lambda_with_args() {
        // Trailing lambda after args: test(1, 2) { }
        let source = r#"
            test(1, 2) {
                print("hello")
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_trailing_lambda_with_semicolon() {
        // Trailing lambda with semicolon: test(1, 2; x, y) { }
        let source = r#"
            test(1, 2; x, y) {
                print(x + y)
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_trailing_lambda_typed_params() {
        // Trailing lambda with typed params: test(1, 2, x: int, y: int) { }
        let source = r#"
            test(1, 2, x: int32, y: int32) {
                print(x + y)
            }
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }

    #[test]
    fn test_parse_function_type() {
        // Function type annotation
        let source = r#"
            let callback: func(int32, int32) int32 = add
        "#;
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(&mut lexer).unwrap();
        let file = parser.parse().unwrap();
        assert!(file.declarations.len() >= 1);
    }
}