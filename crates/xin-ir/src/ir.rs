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
    Object,       // Generic object type for arrays, etc.
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
            IRType::Object => write!(f, "object"),
        }
    }
}

/// Type of operand in string concatenation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcatType {
    String,
    Int,
    Float,
    Bool,
}

/// External function declaration
#[derive(Debug, Clone)]
pub struct ExternFunction {
    pub name: String,
    pub params: Vec<IRType>,
    pub return_type: Option<IRType>,
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

    /// String constant: %result = string_const string_index
    StringConst { result: Value, string_index: usize },

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
        is_extern: bool,
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

    /// String concatenation: %result = concat left, right
    StringConcat {
        result: Value,
        left: Value,
        left_type: ConcatType,
        right: Value,
        right_type: ConcatType,
    },

    /// Convert to string: %result = to_string value
    ToString {
        result: Value,
        value: Value,
        from_type: IRType,
    },

    /// String deallocation: free value
    StringFree {
        value: Value,
    },

    /// Create array: %result = array_new capacity
    ArrayNew {
        result: Value,
        capacity: usize,
    },

    /// Get element: %result = array_get array, index
    ArrayGet {
        result: Value,
        array: Value,
        index: Value,
    },

    /// Set element: array_set array, index, value
    ArraySet {
        array: Value,
        index: Value,
        value: Value,
    },

    /// Append element: array_push array, value
    ArrayPush {
        array: Value,
        value: Value,
    },

    /// Pop element: %result = array_pop array
    ArrayPop {
        result: Value,
        array: Value,
    },

    /// Get length: %result = array_len array
    ArrayLen {
        result: Value,
        array: Value,
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
    pub extern_functions: Vec<ExternFunction>,
    pub strings: Vec<String>,
}

impl IRModule {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            extern_functions: Vec::new(),
            strings: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: IRFunction) {
        self.functions.push(func);
    }

    pub fn add_extern_function(&mut self, func: ExternFunction) {
        self.extern_functions.push(func);
    }

    pub fn add_string(&mut self, s: &str) -> usize {
        // Check if string already exists
        for (i, existing) in self.strings.iter().enumerate() {
            if existing == s {
                return i;
            }
        }
        let index = self.strings.len();
        self.strings.push(s.to_string());
        index
    }
}

impl Default for IRModule {
    fn default() -> Self {
        Self::new()
    }
}