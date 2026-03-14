//! Statement nodes

use xin_diagnostics::SourceSpan;

use crate::{Expr, Type};

/// Statement node
#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: SourceSpan,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

/// Statement kinds
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Variable declaration: let x = 10 or var x = 10
    VarDecl(VarDecl),
    /// Expression statement: foo();
    Expr(Expr),
    /// Return statement: return x
    Return(Option<Expr>),
    /// If statement
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    /// For loop
    For(ForLoop),
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Block: { statements }
    Block(Vec<Stmt>),
}

/// For loop variants
#[derive(Debug, Clone)]
pub enum ForLoop {
    /// C-style for: for (let i = 0; i < 10; i = i + 1) { }
    CStyle {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },
    /// For-in: for (item in list) { }
    ForIn {
        var_name: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    /// Condition-only: for (i < 100) { }
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    /// Infinite loop: for { }
    Infinite {
        body: Vec<Stmt>,
    },
}

/// Variable declaration
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub mutable: bool,
    pub type_annotation: Option<Type>,
    pub value: Option<Expr>,
    pub object_mutable: bool,
}

impl VarDecl {
    pub fn new(name: String, mutable: bool) -> Self {
        Self {
            name,
            mutable,
            type_annotation: None,
            value: None,
            object_mutable: false,
        }
    }
}