//! Type definitions

use std::fmt;

/// Type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// int
    Int,
    /// float
    Float,
    /// bool
    Bool,
    /// string
    String,
    /// void
    Void,
    /// object 类型，表示任意类型的运行时值
    /// 用于混合类型数组的元素类型
    Object,
    /// User-defined type name
    Named(String),
    /// Pointer type: *T or *mut T
    Pointer {
        inner: Box<Type>,
        mutable: bool,
    },
    /// Nullable type: T?
    Nullable(Box<Type>),
    /// Array type: T[]
    Array(Box<Type>),
    /// Generic type: List<T>, Map<K, V>
    Generic {
        name: String,
        args: Vec<Type>,
    },
    /// Function type: func(A, B) R
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Object => write!(f, "object"),
            Type::Named(name) => write!(f, "{}", name),
            Type::Pointer { inner, mutable } => {
                if *mutable {
                    write!(f, "*mut {}", inner)
                } else {
                    write!(f, "*{}", inner)
                }
            }
            Type::Nullable(inner) => write!(f, "{}?", inner),
            Type::Array(inner) => write!(f, "{}[]", inner),
            Type::Generic { name, args } => {
                write!(f, "{}<", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }
            Type::Function { params, return_type } => {
                write!(f, "func(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") {}", return_type)
            }
        }
    }
}