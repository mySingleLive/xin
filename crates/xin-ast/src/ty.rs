//! Type definitions

use std::fmt;

/// Type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // Signed integers
    /// 8-bit signed integer
    Int8,
    /// 16-bit signed integer
    Int16,
    /// 32-bit signed integer
    Int32,
    /// 64-bit signed integer
    Int64,
    /// 128-bit signed integer
    Int128,
    // Unsigned integers
    /// 8-bit unsigned integer
    UInt8,
    /// 16-bit unsigned integer
    UInt16,
    /// 32-bit unsigned integer
    UInt32,
    /// 64-bit unsigned integer
    UInt64,
    /// 128-bit unsigned integer
    UInt128,
    // Floating-point numbers
    /// 8-bit floating-point number
    Float8,
    /// 16-bit floating-point number
    Float16,
    /// 32-bit floating-point number
    Float32,
    /// 64-bit floating-point number
    Float64,
    /// 128-bit floating-point number
    Float128,
    // Character
    /// Unicode character
    Char,
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
            // Signed integers
            Type::Int8 => write!(f, "int8"),
            Type::Int16 => write!(f, "int16"),
            Type::Int32 => write!(f, "int32"),
            Type::Int64 => write!(f, "int64"),
            Type::Int128 => write!(f, "int128"),
            // Unsigned integers
            Type::UInt8 => write!(f, "uint8"),
            Type::UInt16 => write!(f, "uint16"),
            Type::UInt32 => write!(f, "uint32"),
            Type::UInt64 => write!(f, "uint64"),
            Type::UInt128 => write!(f, "uint128"),
            // Floating-point numbers
            Type::Float8 => write!(f, "float8"),
            Type::Float16 => write!(f, "float16"),
            Type::Float32 => write!(f, "float32"),
            Type::Float64 => write!(f, "float64"),
            Type::Float128 => write!(f, "float128"),
            // Character
            Type::Char => write!(f, "char"),
            // Other types
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

impl Type {
    /// Returns true if this is a signed integer type
    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 | Type::Int128
        )
    }

    /// Returns true if this is an unsigned integer type
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 | Type::UInt128
        )
    }

    /// Returns true if this is any integer type (signed or unsigned)
    pub fn is_integer(&self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer()
    }

    /// Returns true if this is a floating-point type
    pub fn is_float(&self) -> bool {
        matches!(
            self,
            Type::Float8 | Type::Float16 | Type::Float32 | Type::Float64 | Type::Float128
        )
    }

    /// Returns true if this is a numeric type (integer or float)
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Returns the bit width for integer types, or None for non-integer types
    pub fn integer_bit_width(&self) -> Option<u32> {
        match self {
            Type::Int8 | Type::UInt8 => Some(8),
            Type::Int16 | Type::UInt16 => Some(16),
            Type::Int32 | Type::UInt32 => Some(32),
            Type::Int64 | Type::UInt64 => Some(64),
            Type::Int128 | Type::UInt128 => Some(128),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_integer_types() {
        assert_eq!(Type::Int8.to_string(), "int8");
        assert_eq!(Type::Int16.to_string(), "int16");
        assert_eq!(Type::Int32.to_string(), "int32");
        assert_eq!(Type::Int64.to_string(), "int64");
        assert_eq!(Type::Int128.to_string(), "int128");
        assert_eq!(Type::UInt8.to_string(), "uint8");
        assert_eq!(Type::UInt16.to_string(), "uint16");
        assert_eq!(Type::UInt32.to_string(), "uint32");
        assert_eq!(Type::UInt64.to_string(), "uint64");
        assert_eq!(Type::UInt128.to_string(), "uint128");
    }

    #[test]
    fn test_new_float_types() {
        assert_eq!(Type::Float8.to_string(), "float8");
        assert_eq!(Type::Float16.to_string(), "float16");
        assert_eq!(Type::Float32.to_string(), "float32");
        assert_eq!(Type::Float64.to_string(), "float64");
        assert_eq!(Type::Float128.to_string(), "float128");
    }

    #[test]
    fn test_char_type() {
        assert_eq!(Type::Char.to_string(), "char");
    }

    #[test]
    fn test_type_helpers() {
        assert!(Type::Int32.is_signed_integer());
        assert!(Type::UInt32.is_unsigned_integer());
        assert!(Type::Int32.is_integer());
        assert!(Type::Float32.is_float());
        assert!(Type::Int32.is_numeric());
        assert_eq!(Type::Int32.integer_bit_width(), Some(32));
        assert_eq!(Type::UInt64.integer_bit_width(), Some(64));
    }

    #[test]
    fn test_type_helpers_negative() {
        assert!(!Type::UInt32.is_signed_integer());
        assert!(!Type::Int32.is_unsigned_integer());
        assert!(!Type::String.is_integer());
        assert!(!Type::Int32.is_float());
        assert!(!Type::String.is_numeric());
    }

    #[test]
    fn test_integer_bit_width_negative() {
        assert_eq!(Type::Float32.integer_bit_width(), None);
        assert_eq!(Type::String.integer_bit_width(), None);
    }

    #[test]
    fn test_boundary_types() {
        assert_eq!(Type::Int128.to_string(), "int128");
        assert_eq!(Type::UInt128.to_string(), "uint128");
        assert_eq!(Type::Float128.to_string(), "float128");
        assert_eq!(Type::Int128.integer_bit_width(), Some(128));
    }
}