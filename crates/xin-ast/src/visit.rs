//! AST visitor pattern

use crate::{Decl, DeclKind, Expr, ExprKind, Stmt, StmtKind};

/// AST visitor trait
pub trait Visitor {
    fn visit_decl(&mut self, decl: &Decl) {
        match &decl.kind {
            DeclKind::Func(f) => self.visit_func_decl(f),
            DeclKind::Struct(s) => self.visit_struct_decl(s),
            DeclKind::Interface(i) => self.visit_interface_decl(i),
            DeclKind::Import(i) => self.visit_import_decl(i),
        }
    }

    fn visit_func_decl(&mut self, _decl: &crate::FuncDecl) {}
    fn visit_struct_decl(&mut self, _decl: &crate::StructDecl) {}
    fn visit_interface_decl(&mut self, _decl: &crate::InterfaceDecl) {}
    fn visit_import_decl(&mut self, _decl: &crate::ImportDecl) {}

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::VarDecl(v) => self.visit_var_decl(v),
            StmtKind::Expr(e) => self.visit_expr(e),
            StmtKind::Return(e) => self.visit_return(e.as_ref()),
            StmtKind::If { condition, then_block, else_block } => {
                self.visit_expr(condition);
                for s in then_block {
                    self.visit_stmt(s);
                }
                if let Some(else_block) = else_block {
                    for s in else_block {
                        self.visit_stmt(s);
                    }
                }
            }
            StmtKind::For(for_loop) => self.visit_for_loop(for_loop),
            StmtKind::Break => {}
            StmtKind::Continue => {}
            StmtKind::Block(stmts) => {
                for s in stmts {
                    self.visit_stmt(s);
                }
            }
        }
    }

    fn visit_var_decl(&mut self, decl: &crate::VarDecl) {
        if let Some(value) = &decl.value {
            self.visit_expr(value);
        }
    }

    fn visit_return(&mut self, expr: Option<&Expr>) {
        if let Some(e) = expr {
            self.visit_expr(e);
        }
    }

    fn visit_for_loop(&mut self, _for_loop: &crate::ForLoop) {}

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::StringLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Null
            | ExprKind::Ident(_) => {}
            ExprKind::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            ExprKind::Unary { operand, .. } => {
                self.visit_expr(operand);
            }
            ExprKind::Call { callee, args } => {
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprKind::MethodCall { object, args, .. } => {
                self.visit_expr(object);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprKind::FieldAccess { object, .. } => {
                self.visit_expr(object);
            }
            ExprKind::SafeAccess { object, .. } => {
                self.visit_expr(object);
            }
            ExprKind::Elvis { left, right } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            ExprKind::ForceUnwrap(e) => {
                self.visit_expr(e);
            }
            ExprKind::Index { object, index } => {
                self.visit_expr(object);
                self.visit_expr(index);
            }
            ExprKind::StructInstance { fields, .. } => {
                for (_, value) in fields {
                    self.visit_expr(value);
                }
            }
            ExprKind::ArrayLiteral(elements) => {
                for e in elements {
                    self.visit_expr(e);
                }
            }
            ExprKind::MapLiteral(entries) => {
                for (k, v) in entries {
                    self.visit_expr(k);
                    self.visit_expr(v);
                }
            }
            ExprKind::Lambda { body, .. } => match body {
                crate::LambdaBody::Expr(e) => self.visit_expr(e),
                crate::LambdaBody::Block(stmts) => {
                    for s in stmts {
                        self.visit_stmt(s);
                    }
                }
            },
            ExprKind::If { condition, then_branch, else_branch } => {
                self.visit_expr(condition);
                self.visit_expr(then_branch);
                if let Some(else_branch) = else_branch {
                    self.visit_expr(else_branch);
                }
            }
            ExprKind::Conditional { condition, then_expr, else_expr } => {
                self.visit_expr(condition);
                self.visit_expr(then_expr);
                self.visit_expr(else_expr);
            }
            ExprKind::Assignment { target, value } => {
                self.visit_expr(target);
                self.visit_expr(value);
            }
            ExprKind::Move(e) => {
                self.visit_expr(e);
            }
            ExprKind::Cast { expr, .. } => {
                self.visit_expr(expr);
            }
            ExprKind::TemplateLiteral(parts) => {
                for part in parts {
                    if let crate::TemplatePart::Expr(e) = part {
                        self.visit_expr(e);
                    }
                }
            }
        }
    }
}