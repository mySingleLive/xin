//! IR definitions

use std::fmt;

/// IR Value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value(pub String);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// IR Types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRType {
    I64,
    F64,
    Bool,
    String,
    Void,
    Ptr(String), // Pointer to named type
}

impl fmt::Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRType::I64 => write!(f, "i64"),
            IRType::F64 => write!(f, "f64"),
            IRType::Bool => write!(f, "bool"),
            IRType::String => write!(f, "string"),
            IRType::Void => write!(f, "void"),
            IRType::Ptr(inner) => write!(f, "ptr<{}>", inner),
        }
    }
}

/// IR Instructions
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Allocate local variable: %v = alloca type
    Alloca { result: Value, ty: IRType },

    /// Store value: store value, ptr
    Store { value: Value, ptr: Value },

    /// Load value: %result = load ptr
    Load { result: Value, ptr: Value },

    /// Constant: %result = const value
    Const { result: Value, value: String, ty: IRType },

    /// Binary operation: %result = op left, right
    Binary {
        result: Value,
        op: BinOp,
        left: Value,
        right: Value,
    },

    /// Function call: %result = call func(args...)
    Call {
        result: Option<Value>,
        func: String,
        args: Vec<Value>,
    },

    /// Return: ret value
    Return(Option<Value>),

    /// Jump: br label
    Jump(String),

    /// Conditional branch: br cond, then_label, else_label
    Branch {
        cond: Value,
        then_label: String,
        else_label: String,
    },

    /// Label: label:
    Label(String),

    /// Phi: %result = phi [val1, label1], [val2, label2]
    Phi {
        result: Value,
        incoming: Vec<(Value, String)>,
    },
}

/// Binary operations in IR
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

/// IR Function
#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<(String, IRType)>,
    pub return_type: IRType,
    pub instructions: Vec<Instruction>,
}

/// IR Module
#[derive(Debug, Clone)]
pub struct IRModule {
    pub functions: Vec<IRFunction>,
}

impl IRModule {
    pub fn new() -> Self {
        Self { functions: Vec::new() }
    }

    pub fn add_function(&mut self, func: IRFunction) {
        self.functions.push(func);
    }
}

impl Default for IRModule {
    fn default() -> Self {
        Self::new()
    }
}