//! Expression nodes

use xin_diagnostics::SourceSpan;

use crate::Type;

/// Expression node
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

impl Expr {
    pub fn new(kind: ExprKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

/// Expression kinds
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Integer literal: 42
    IntLiteral(i64),
    /// Float literal: 3.14
    FloatLiteral(f64),
    /// String literal: "hello"
    StringLiteral(String),
    /// Template string: `hello {name}`
    TemplateLiteral(Vec<TemplatePart>),
    /// Boolean literal: true, false
    BoolLiteral(bool),
    /// Null literal
    Null,
    /// Identifier: x
    Ident(String),
    /// Binary operation: a + b
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation: -x, !x
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// Function call: foo(a, b)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Method call: obj.method(a, b)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// Field access: obj.field
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    /// Safe navigation: obj?.field
    SafeAccess {
        object: Box<Expr>,
        field: String,
    },
    /// Elvis operator: x ?? default
    Elvis {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Force unwrap: x!!
    ForceUnwrap(Box<Expr>),
    /// Index access: arr[i]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    /// Struct instantiation: User { name: "a", age: 10 }
    StructInstance {
        name: String,
        fields: Vec<(String, Expr)>,
        mutable: bool,
    },
    /// Array literal: [1, 2, 3]
    ArrayLiteral(Vec<Expr>),
    /// Map literal: { "a": 1, "b": 2 }
    MapLiteral(Vec<(Expr, Expr)>),
    /// Lambda: (a, b) -> a + b
    Lambda {
        params: Vec<LambdaParam>,
        return_type: Option<Type>,
        body: LambdaBody,
    },
    /// If expression: if a > 0 { "yes" } else { "no" }
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    /// Conditional expression (ternary): a > b ? a : b
    Conditional {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    /// Assignment: x = 10
    Assignment {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    /// Move expression: move x
    Move(Box<Expr>),
    /// Type cast: int(x)
    Cast {
        expr: Box<Expr>,
        target_type: Type,
    },
}

/// Lambda parameter
#[derive(Debug, Clone)]
pub struct LambdaParam {
    pub name: String,
    pub type_annotation: Option<Type>,
}

/// Lambda body
#[derive(Debug, Clone)]
pub enum LambdaBody {
    /// Expression body: -> a + b
    Expr(Box<Expr>),
    /// Block body: -> { return a + b }
    Block(Vec<crate::Stmt>),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Template string part
#[derive(Debug, Clone)]
pub enum TemplatePart {
    /// Plain text
    Text(String),
    /// Embedded expression
    Expr(Box<Expr>),
}