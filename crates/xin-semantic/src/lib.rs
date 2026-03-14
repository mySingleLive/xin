//! Semantic analysis for Xin

mod error;
mod scope;
mod symbol;
mod type_check;

pub use error::SemanticError;
pub use scope::ScopeStack;
pub use symbol::{Symbol, SymbolKind, SymbolTable};
pub use type_check::TypeChecker;