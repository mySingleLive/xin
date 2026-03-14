//! Scope management

use std::collections::HashMap;

use crate::{Symbol, SymbolTable};

/// Scope information
#[derive(Debug, Clone)]
pub struct Scope {
    pub level: usize,
    pub parent: Option<usize>,
    pub locals: HashMap<String, usize>, // name -> symbol index
}

/// Scope stack for managing nested scopes
#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
    current: usize,
    symbols: SymbolTable,
}

impl ScopeStack {
    pub fn new() -> Self {
        let global = Scope {
            level: 0,
            parent: None,
            locals: HashMap::new(),
        };
        Self {
            scopes: vec![global],
            current: 0,
            symbols: SymbolTable::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        let new_scope = Scope {
            level: self.scopes.len(),
            parent: Some(self.current),
            locals: HashMap::new(),
        };
        self.scopes.push(new_scope);
        self.current = self.scopes.len() - 1;
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }

    pub fn define(&mut self, name: &str, symbol: Symbol) {
        let idx = self.symbols.symbols.len();
        self.symbols.define(symbol);
        self.scopes[self.current].locals.insert(name.to_string(), idx);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut scope_idx = self.current;

        loop {
            if let Some(&idx) = self.scopes[scope_idx].locals.get(name) {
                return Some(&self.symbols.symbols[idx]);
            }

            scope_idx = match self.scopes[scope_idx].parent {
                Some(p) => p,
                None => return None,
            };
        }
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        let mut scope_idx = self.current;

        loop {
            if let Some(&idx) = self.scopes[scope_idx].locals.get(name) {
                return Some(&mut self.symbols.symbols[idx]);
            }

            scope_idx = match self.scopes[scope_idx].parent {
                Some(p) => p,
                None => return None,
            };
        }
    }

    pub fn current_level(&self) -> usize {
        self.current
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}