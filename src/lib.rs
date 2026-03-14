//! Xin Programming Language Compiler
//!
//! A statically-typed, compiled programming language with memory safety
//! without manual memory management or runtime GC.

pub mod compiler;

pub use xin_ast as ast;
pub use xin_lexer as lexer;
pub use xin_parser as parser;
pub use xin_semantic as semantic;
pub use xin_ir as ir;
pub use xin_codegen as codegen;
pub use xin_diagnostics as diagnostics;