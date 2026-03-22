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
    // Signed integers
    I8,
    I16,
    I32,
    I64,
    I128,
    // Unsigned integers
    U8,
    U16,
    U32,
    U64,
    U128,
    // Floats
    F8,
    F16,
    F32,
    F64,
    F128,
    // Other types
    Char,
    Bool,
    String,
    Void,
    Ptr(String), // Pointer to named type
    Object,       // Generic object type for arrays, etc.
}

impl fmt::Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Signed integers
            IRType::I8 => write!(f, "i8"),
            IRType::I16 => write!(f, "i16"),
            IRType::I32 => write!(f, "i32"),
            IRType::I64 => write!(f, "i64"),
            IRType::I128 => write!(f, "i128"),
            // Unsigned integers
            IRType::U8 => write!(f, "u8"),
            IRType::U16 => write!(f, "u16"),
            IRType::U32 => write!(f, "u32"),
            IRType::U64 => write!(f, "u64"),
            IRType::U128 => write!(f, "u128"),
            // Floats
            IRType::F8 => write!(f, "f8"),
            IRType::F16 => write!(f, "f16"),
            IRType::F32 => write!(f, "f32"),
            IRType::F64 => write!(f, "f64"),
            IRType::F128 => write!(f, "f128"),
            // Other types
            IRType::Char => write!(f, "char"),
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

    /// Type cast: %result = cast value, target_type
    TypeCast {
        result: Value,
        value: Value,
        from_type: IRType,
        to_type: IRType,
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

    /// Break out of loop
    Break,

    /// Continue to next iteration
    Continue,
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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Alloca { result, ty } => write!(f, "{} = alloca {}", result, ty),
            Instruction::Store { value, ptr } => write!(f, "store {}, {}", value, ptr),
            Instruction::Load { result, ptr } => write!(f, "{} = load {}", result, ptr),
            Instruction::Const { result, value, ty } => write!(f, "{} = const {} {}", result, value, ty),
            Instruction::StringConst { result, string_index } => {
                write!(f, "{} = string_const {}", result, string_index)
            }
            Instruction::Binary {
                result,
                op,
                left,
                right,
            } => {
                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "div",
                    BinOp::Mod => "mod",
                    BinOp::Eq => "eq",
                    BinOp::Ne => "ne",
                    BinOp::Lt => "lt",
                    BinOp::Gt => "gt",
                    BinOp::Le => "le",
                    BinOp::Ge => "ge",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                };
                write!(f, "{} = {} {}, {}", result, op_str, left, right)
            }
            Instruction::Call {
                result,
                func,
                args,
                is_extern,
            } => {
                if let Some(res) = result {
                    write!(f, "{} = call {}({})", res, func, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))?;
                } else {
                    write!(f, "call {}({})", func, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))?;
                }
                if *is_extern {
                    write!(f, " [extern]")?;
                }
                Ok(())
            }
            Instruction::Return(val) => {
                if let Some(v) = val {
                    write!(f, "ret {}", v)
                } else {
                    write!(f, "ret")
                }
            }
            Instruction::Jump(label) => write!(f, "br {}", label),
            Instruction::Branch {
                cond,
                then_label,
                else_label,
            } => write!(f, "br {}, {}, {}", cond, then_label, else_label),
            Instruction::Label(label) => write!(f, "{}:", label),
            Instruction::Phi { result, incoming } => {
                write!(
                    f,
                    "{} = phi {}",
                    result,
                    incoming
                        .iter()
                        .map(|(v, l)| format!("[{}, {}]", v, l))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Instruction::StringConcat {
                result,
                left,
                left_type,
                right,
                right_type,
            } => {
                let left_ty = match left_type {
                    ConcatType::String => "string",
                    ConcatType::Int => "int",
                    ConcatType::Float => "float",
                    ConcatType::Bool => "bool",
                };
                let right_ty = match right_type {
                    ConcatType::String => "string",
                    ConcatType::Int => "int",
                    ConcatType::Float => "float",
                    ConcatType::Bool => "bool",
                };
                write!(f, "{} = concat {}:{}, {}:{}", result, left, left_ty, right, right_ty)
            }
            Instruction::ToString {
                result,
                value,
                from_type,
            } => write!(f, "{} = to_string {} [{}]", result, value, from_type),
            Instruction::TypeCast {
                result,
                value,
                from_type,
                to_type,
            } => write!(f, "{} = cast {} from {} to {}", result, value, from_type, to_type),
            Instruction::StringFree { value } => write!(f, "free {}", value),
            Instruction::ArrayNew { result, capacity } => {
                write!(f, "{} = array_new {}", result, capacity)
            }
            Instruction::ArrayGet {
                result,
                array,
                index,
            } => write!(f, "{} = array_get {}, {}", result, array, index),
            Instruction::ArraySet {
                array,
                index,
                value,
            } => write!(f, "array_set {}, {}, {}", array, index, value),
            Instruction::ArrayPush { array, value } => write!(f, "array_push {}, {}", array, value),
            Instruction::ArrayPop { result, array } => write!(f, "{} = array_pop {}", result, array),
            Instruction::ArrayLen { result, array } => write!(f, "{} = array_len {}", result, array),
            Instruction::Break => write!(f, "break"),
            Instruction::Continue => write!(f, "continue"),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_break_continue_instructions() {
        // 测试 Break 指令存在
        let brk = Instruction::Break;
        assert!(matches!(brk, Instruction::Break));

        // 测试 Continue 指令存在
        let cnt = Instruction::Continue;
        assert!(matches!(cnt, Instruction::Continue));
    }
}