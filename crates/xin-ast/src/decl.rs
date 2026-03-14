//! Declaration nodes

use xin_diagnostics::SourceSpan;

use crate::{Expr, Stmt, Type};

/// Top-level declaration
#[derive(Debug, Clone)]
pub struct Decl {
    pub kind: DeclKind,
    pub span: SourceSpan,
}

impl Decl {
    pub fn new(kind: DeclKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

/// Declaration kinds
#[derive(Debug, Clone)]
pub enum DeclKind {
    /// Function declaration
    Func(FuncDecl),
    /// Struct declaration
    Struct(StructDecl),
    /// Interface declaration
    Interface(InterfaceDecl),
    /// Import declaration
    Import(ImportDecl),
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub return_type: Option<Type>,
    pub body: FuncBody,
    pub is_public: bool,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct FuncParam {
    pub name: String,
    pub type_annotation: Type,
    pub mutable: bool,
}

/// Function body
#[derive(Debug, Clone)]
pub enum FuncBody {
    /// Block body: { statements }
    Block(Vec<Stmt>),
    /// Expression body: -> expr
    Expr(Expr),
}

/// Struct declaration
#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<StructField>,
    pub methods: Vec<FuncDecl>,
    pub implements: Option<String>,
    pub is_public: bool,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_annotation: Type,
    pub is_public: bool,
}

/// Interface declaration
#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub methods: Vec<InterfaceMethod>,
    pub is_public: bool,
}

/// Interface method
#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub return_type: Option<Type>,
    pub is_mutating: bool,
}

/// Import declaration
#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub module: String,
    pub items: Option<Vec<ImportItem>>,
}

/// Import item
#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

/// A complete source file
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub declarations: Vec<Decl>,
}