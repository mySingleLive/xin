//! Symbol definitions

use xin_ast::Type;

/// Symbol kinds
#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable {
        mutable: bool,
        type_annotation: Type,
        moved: bool,
    },
    Function {
        params: Vec<(String, Type, bool)>, // (name, type, mutable)
        return_type: Type,
    },
    Struct {
        fields: Vec<(String, Type, bool)>, // (name, type, public)
        methods: Vec<crate::symbol::StructMethod>,
    },
    Interface {
        methods: Vec<(String, Vec<(String, Type, bool)>, Option<Type>, bool)>, // (name, params, return_type, is_mutating)
    },
    TypeAlias(Type),
}

/// Struct method representation
#[derive(Debug, Clone)]
pub struct StructMethod {
    pub name: String,
    pub params: Vec<(String, Type, bool)>,
    pub return_type: Option<Type>,
}

/// A symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub scope_level: usize,
}

impl Symbol {
    pub fn variable(name: String, mutable: bool, type_annotation: Type, scope_level: usize) -> Self {
        Self {
            name,
            kind: SymbolKind::Variable {
                mutable,
                type_annotation,
                moved: false,
            },
            scope_level,
        }
    }

    pub fn function(
        name: String,
        params: Vec<(String, Type, bool)>,
        return_type: Type,
        scope_level: usize,
    ) -> Self {
        Self {
            name,
            kind: SymbolKind::Function { params, return_type },
            scope_level,
        }
    }

    pub fn get_type(&self) -> Option<Type> {
        match &self.kind {
            SymbolKind::Variable { type_annotation, .. } => Some(type_annotation.clone()),
            SymbolKind::Function { return_type, .. } => Some(return_type.clone()),
            _ => None,
        }
    }

    pub fn is_mutable(&self) -> bool {
        match &self.kind {
            SymbolKind::Variable { mutable, .. } => *mutable,
            _ => false,
        }
    }

    pub fn is_moved(&self) -> bool {
        match &self.kind {
            SymbolKind::Variable { moved, .. } => *moved,
            _ => false,
        }
    }

    pub fn mark_moved(&mut self) {
        if let SymbolKind::Variable { moved, .. } = &mut self.kind {
            *moved = true;
        }
    }
}

/// Symbol table
#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { symbols: Vec::new() }
    }

    pub fn define(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    pub fn lookup(&self, name: &str, scope_level: usize) -> Option<&Symbol> {
        self.symbols
            .iter()
            .rev()
            .find(|s| s.name == name && s.scope_level <= scope_level)
    }

    pub fn lookup_mut(&mut self, name: &str, scope_level: usize) -> Option<&mut Symbol> {
        self.symbols
            .iter_mut()
            .rev()
            .find(|s| s.name == name && s.scope_level <= scope_level)
    }

    pub fn remove_scope(&mut self, scope_level: usize) {
        self.symbols.retain(|s| s.scope_level != scope_level);
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}