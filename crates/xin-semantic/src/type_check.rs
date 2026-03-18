//! Type checking

use xin_ast::*;
use xin_diagnostics::Diagnostic;

use crate::{ScopeStack, SemanticError, Symbol, SymbolKind};

/// Type checker
pub struct TypeChecker {
    scopes: ScopeStack,
    diagnostics: Vec<Diagnostic>,
    current_function_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            diagnostics: Vec::new(),
            current_function_return_type: None,
        }
    }

    pub fn check(&mut self, file: &SourceFile) -> Result<(), Vec<Diagnostic>> {
        // Register built-in functions
        self.register_builtins();

        // First pass: collect all top-level declarations
        for decl in &file.declarations {
            self.collect_declaration(decl);
        }

        // Second pass: type check all declarations
        for decl in &file.declarations {
            if let Err(e) = self.check_declaration(decl) {
                self.diagnostics.push(e.into());
            }
        }

        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self.diagnostics.clone())
        }
    }

    /// Register built-in functions like println, print
    fn register_builtins(&mut self) {
        // println: accepts any type and returns void
        let println_symbol = Symbol {
            name: "println".to_string(),
            kind: SymbolKind::Function {
                params: vec![("_".to_string(), Type::String, false)], // Accept one argument of any type
                return_type: Type::Void,
            },
            scope_level: 0,
        };
        self.scopes.define("println", println_symbol);

        // print: accepts any type and returns void
        let print_symbol = Symbol {
            name: "print".to_string(),
            kind: SymbolKind::Function {
                params: vec![("_".to_string(), Type::String, false)],
                return_type: Type::Void,
            },
            scope_level: 0,
        };
        self.scopes.define("print", print_symbol);

        // printf: accepts format string and variable args, returns void
        let printf_symbol = Symbol {
            name: "printf".to_string(),
            kind: SymbolKind::Function {
                params: vec![("format".to_string(), Type::String, false)],
                return_type: Type::Void,
            },
            scope_level: 0,
        };
        self.scopes.define("printf", printf_symbol);
    }

    fn collect_declaration(&mut self, decl: &Decl) {
        match &decl.kind {
            DeclKind::Func(f) => {
                let params: Vec<(String, Type, bool)> = f
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.type_annotation.clone(), p.mutable))
                    .collect();
                let return_type = f.return_type.clone().unwrap_or(Type::Void);
                let symbol = Symbol::function(f.name.clone(), params, return_type, 0);
                self.scopes.define(&f.name, symbol);
            }
            DeclKind::Struct(s) => {
                let fields: Vec<(String, Type, bool)> = s
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.type_annotation.clone(), f.is_public))
                    .collect();
                let methods: Vec<crate::symbol::StructMethod> = s
                    .methods
                    .iter()
                    .map(|m| crate::symbol::StructMethod {
                        name: m.name.clone(),
                        params: m.params.iter().map(|p| (p.name.clone(), p.type_annotation.clone(), p.mutable)).collect(),
                        return_type: m.return_type.clone(),
                    })
                    .collect();
                let symbol = Symbol {
                    name: s.name.clone(),
                    kind: SymbolKind::Struct { fields, methods },
                    scope_level: 0,
                };
                self.scopes.define(&s.name, symbol);
            }
            DeclKind::Interface(i) => {
                let methods: Vec<_> = i
                    .methods
                    .iter()
                    .map(|m| {
                        let params: Vec<_> = m
                            .params
                            .iter()
                            .map(|p| (p.name.clone(), p.type_annotation.clone(), p.mutable))
                            .collect();
                        (m.name.clone(), params, m.return_type.clone(), m.is_mutating)
                    })
                    .collect();
                let symbol = Symbol {
                    name: i.name.clone(),
                    kind: SymbolKind::Interface { methods },
                    scope_level: 0,
                };
                self.scopes.define(&i.name, symbol);
            }
            DeclKind::Import(_) => {}
        }
    }

    fn check_declaration(&mut self, decl: &Decl) -> Result<(), SemanticError> {
        match &decl.kind {
            DeclKind::Func(f) => self.check_func_decl(f),
            DeclKind::Struct(s) => self.check_struct_decl(s),
            DeclKind::Interface(i) => self.check_interface_decl(i),
            DeclKind::Import(_) => Ok(()),
        }
    }

    fn check_func_decl(&mut self, func: &FuncDecl) -> Result<(), SemanticError> {
        self.scopes.enter_scope();

        // Store return type for checking return statements
        self.current_function_return_type = func.return_type.clone();

        // Add parameters to scope
        for param in &func.params {
            let symbol = Symbol::variable(
                param.name.clone(),
                param.mutable,
                param.type_annotation.clone(),
                self.scopes.current_level(),
            );
            self.scopes.define(&param.name, symbol);
        }

        // Check body
        match &func.body {
            FuncBody::Block(stmts) => {
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
            }
            FuncBody::Expr(expr) => {
                let expr_type = self.check_expr(expr)?;
                if let Some(expected) = &func.return_type {
                    if !self.types_compatible(expected, &expr_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected.clone(),
                            found: expr_type,
                        });
                    }
                }
            }
        }

        self.scopes.exit_scope();
        Ok(())
    }

    fn check_struct_decl(&mut self, _struct: &StructDecl) -> Result<(), SemanticError> {
        // Check field types
        for field in &_struct.fields {
            self.check_type_exists(&field.type_annotation)?;
        }

        // Check methods
        for method in &_struct.methods {
            self.check_func_decl(method)?;
        }

        Ok(())
    }

    fn check_interface_decl(&mut self, _interface: &InterfaceDecl) -> Result<(), SemanticError> {
        // Check method signatures
        for method in &_interface.methods {
            for param in &method.params {
                self.check_type_exists(&param.type_annotation)?;
            }
            if let Some(ret) = &method.return_type {
                self.check_type_exists(ret)?;
            }
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match &stmt.kind {
            StmtKind::VarDecl(var) => {
                if let Some(value) = &var.value {
                    let value_type = self.check_expr(value)?;

                    let var_type = if let Some(ty) = &var.type_annotation {
                        self.check_type_exists(ty)?;
                        // Check type compatibility
                        if !self.types_compatible(ty, &value_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: ty.clone(),
                                found: value_type,
                            });
                        }
                        ty.clone()
                    } else {
                        value_type
                    };

                    let symbol = Symbol::variable(
                        var.name.clone(),
                        var.mutable,
                        var_type,
                        self.scopes.current_level(),
                    );
                    self.scopes.define(&var.name, symbol);
                } else if let Some(ty) = &var.type_annotation {
                    self.check_type_exists(ty)?;
                    let symbol = Symbol::variable(
                        var.name.clone(),
                        var.mutable,
                        ty.clone(),
                        self.scopes.current_level(),
                    );
                    self.scopes.define(&var.name, symbol);
                }
            }
            StmtKind::Expr(expr) => {
                self.check_expr(expr)?;
            }
            StmtKind::Return(value) => {
                let return_type = value
                    .as_ref()
                    .map(|e| self.check_expr(e))
                    .transpose()?
                    .unwrap_or(Type::Void);

                if let Some(expected) = &self.current_function_return_type {
                    if !self.types_compatible(expected, &return_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected.clone(),
                            found: return_type,
                        });
                    }
                }
            }
            StmtKind::If { condition, then_block, else_block } => {
                let cond_type = self.check_expr(condition)?;
                if cond_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: cond_type,
                    });
                }

                self.scopes.enter_scope();
                for stmt in then_block {
                    self.check_stmt(stmt)?;
                }
                self.scopes.exit_scope();

                if let Some(else_block) = else_block {
                    self.scopes.enter_scope();
                    for stmt in else_block {
                        self.check_stmt(stmt)?;
                    }
                    self.scopes.exit_scope();
                }
            }
            StmtKind::For(for_loop) => {
                self.scopes.enter_scope();
                match for_loop {
                    ForLoop::CStyle { init, condition, update, body } => {
                        if let Some(init) = init {
                            self.check_stmt(init)?;
                        }
                        if let Some(cond) = condition {
                            let cond_type = self.check_expr(cond)?;
                            if cond_type != Type::Bool {
                                return Err(SemanticError::TypeMismatch {
                                    expected: Type::Bool,
                                    found: cond_type,
                                });
                            }
                        }
                        if let Some(update) = update {
                            self.check_expr(update)?;
                        }
                        for stmt in body {
                            self.check_stmt(stmt)?;
                        }
                    }
                    ForLoop::ForIn { var_name, iterable, body } => {
                        let iter_type = self.check_expr(iterable)?;
                        // TODO: Check that iter_type is iterable

                        // Infer element type from iterable
                        let elem_type = match &iter_type {
                            Type::Array(inner) => (**inner).clone(),
                            Type::Generic { name, args } if name == "List" && !args.is_empty() => {
                                args[0].clone()
                            }
                            _ => Type::Void, // Unknown
                        };

                        let symbol = Symbol::variable(
                            var_name.clone(),
                            true,
                            elem_type,
                            self.scopes.current_level(),
                        );
                        self.scopes.define(var_name, symbol);

                        for stmt in body {
                            self.check_stmt(stmt)?;
                        }
                    }
                    ForLoop::While { condition, body } => {
                        let cond_type = self.check_expr(condition)?;
                        if cond_type != Type::Bool {
                            return Err(SemanticError::TypeMismatch {
                                expected: Type::Bool,
                                found: cond_type,
                            });
                        }
                        for stmt in body {
                            self.check_stmt(stmt)?;
                        }
                    }
                    ForLoop::Infinite { body } => {
                        for stmt in body {
                            self.check_stmt(stmt)?;
                        }
                    }
                }
                self.scopes.exit_scope();
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Block(stmts) => {
                self.scopes.enter_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                self.scopes.exit_scope();
            }
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<Type, SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Ok(Type::Int),
            ExprKind::FloatLiteral(_) => Ok(Type::Float),
            ExprKind::StringLiteral(_) => Ok(Type::String),
            ExprKind::BoolLiteral(_) => Ok(Type::Bool),
            ExprKind::Null => Ok(Type::Nullable(Box::new(Type::Void))),

            ExprKind::Ident(name) => {
                let symbol = self.scopes.lookup(name).ok_or_else(|| {
                    SemanticError::UndefinedVariable(name.clone())
                })?;

                if symbol.is_moved() {
                    return Err(SemanticError::UseAfterMove(name.clone()));
                }

                symbol.get_type().ok_or_else(|| {
                    SemanticError::UndefinedVariable(name.clone())
                })
            }

            ExprKind::Binary { op, left, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // String concatenation: if either side is string, result is string
                        if left_type == Type::String || right_type == Type::String {
                            if *op == BinOp::Add {
                                // Allow string concatenation with any basic type
                                match (&left_type, &right_type) {
                                    (Type::String, Type::String)
                                    | (Type::String, Type::Int)
                                    | (Type::String, Type::Float)
                                    | (Type::String, Type::Bool)
                                    | (Type::Int, Type::String)
                                    | (Type::Float, Type::String)
                                    | (Type::Bool, Type::String) => {
                                        return Ok(Type::String);
                                    }
                                    _ => {
                                        // Determine which type doesn't support string concatenation
                                        let unsupported_type = match (&left_type, &right_type) {
                                            (Type::String, _) => right_type.clone(),
                                            (_, Type::String) => left_type.clone(),
                                            _ => right_type.clone(),
                                        };
                                        return Err(SemanticError::TypeMismatch {
                                            expected: Type::String,
                                            found: unsupported_type,
                                        });
                                    }
                                }
                            }
                        }
                        // Numeric operations
                        if left_type == Type::Int && right_type == Type::Int {
                            Ok(Type::Int)
                        } else if left_type == Type::Float || right_type == Type::Float {
                            Ok(Type::Float)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: left_type.clone(),
                                found: right_type,
                            })
                        }
                    }
                    BinOp::Eq | BinOp::Ne => Ok(Type::Bool),
                    BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        if left_type == Type::Int || left_type == Type::Float {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: Type::Int,
                                found: left_type,
                            })
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if left_type != Type::Bool {
                            return Err(SemanticError::TypeMismatch {
                                expected: Type::Bool,
                                found: left_type,
                            });
                        }
                        if right_type != Type::Bool {
                            return Err(SemanticError::TypeMismatch {
                                expected: Type::Bool,
                                found: right_type,
                            });
                        }
                        Ok(Type::Bool)
                    }
                }
            }

            ExprKind::Unary { op, operand } => {
                let operand_type = self.check_expr(operand)?;
                match op {
                    UnaryOp::Neg => {
                        if operand_type == Type::Int || operand_type == Type::Float {
                            Ok(operand_type)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: Type::Int,
                                found: operand_type,
                            })
                        }
                    }
                    UnaryOp::Not => {
                        if operand_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: Type::Bool,
                                found: operand_type,
                            })
                        }
                    }
                }
            }

            ExprKind::Call { callee, args } => {
                // Check if callee is a function name
                if let ExprKind::Ident(name) = &callee.kind {
                    // Check for built-in functions that accept any type
                    if name == "println" || name == "print" {
                        // These accept any single argument
                        if args.len() != 1 {
                            return Err(SemanticError::WrongNumberOfArguments {
                                expected: 1,
                                found: args.len(),
                            });
                        }
                        self.check_expr(&args[0])?;
                        return Ok(Type::Void);
                    }

                    // Handle printf with format string validation
                    if name == "printf" {
                        if args.is_empty() {
                            return Err(SemanticError::WrongNumberOfArguments {
                                expected: 1,
                                found: 0,
                            });
                        }
                        // Check first argument is a string literal
                        if let ExprKind::StringLiteral(format_str) = &args[0].kind {
                            let expected_types = self.parse_printf_format(format_str)?;
                            if args.len() - 1 != expected_types.len() {
                                return Err(SemanticError::PrintfArgumentCountMismatch {
                                    expected: expected_types.len(),
                                    found: args.len() - 1,
                                });
                            }
                            for (arg, expected_type) in args[1..].iter().zip(expected_types.iter()) {
                                let arg_type = self.check_expr(arg)?;
                                if !self.types_compatible(expected_type, &arg_type) {
                                    return Err(SemanticError::PrintfArgumentTypeMismatch {
                                        expected: expected_type.clone(),
                                        found: arg_type,
                                    });
                                }
                            }
                            return Ok(Type::Void);
                        } else {
                            // Non-literal format string, check it's a string type
                            let format_type = self.check_expr(&args[0])?;
                            if format_type != Type::String {
                                return Err(SemanticError::TypeMismatch {
                                    expected: Type::String,
                                    found: format_type,
                                });
                            }
                            // Can't validate format at compile time, just check remaining args
                            for arg in &args[1..] {
                                self.check_expr(arg)?;
                            }
                            return Ok(Type::Void);
                        }
                    }

                    // Clone the function info if found
                    let func_info = self.scopes.lookup(name).and_then(|symbol| {
                        if let SymbolKind::Function { params, return_type } = &symbol.kind {
                            Some((params.clone(), return_type.clone()))
                        } else {
                            None
                        }
                    });

                    if let Some((params, return_type)) = func_info {
                        if args.len() != params.len() {
                            return Err(SemanticError::WrongNumberOfArguments {
                                expected: params.len(),
                                found: args.len(),
                            });
                        }

                        for (arg, (_, param_type, _)) in args.iter().zip(params.iter()) {
                            let arg_type = self.check_expr(arg)?;
                            if !self.types_compatible(param_type, &arg_type) {
                                return Err(SemanticError::TypeMismatch {
                                    expected: param_type.clone(),
                                    found: arg_type,
                                });
                            }
                        }

                        return Ok(return_type);
                    }
                }

                // General case: check callee type
                let callee_type = self.check_expr(callee)?;

                match callee_type {
                    Type::Function { params, return_type } => {
                        if args.len() != params.len() {
                            return Err(SemanticError::WrongNumberOfArguments {
                                expected: params.len(),
                                found: args.len(),
                            });
                        }

                        for (arg, param_type) in args.iter().zip(params.iter()) {
                            let arg_type = self.check_expr(arg)?;
                            if !self.types_compatible(param_type, &arg_type) {
                                return Err(SemanticError::TypeMismatch {
                                    expected: param_type.clone(),
                                    found: arg_type,
                                });
                            }
                        }

                        Ok(*return_type)
                    }
                    _ => Err(SemanticError::UndefinedFunction(format!("{:?}", callee))),
                }
            }

            ExprKind::MethodCall { object, method, args } => {
                let obj_type = self.check_expr(object)?;

                // Look up method in struct
                let type_name = match &obj_type {
                    Type::Named(name) => name.clone(),
                    Type::Pointer { inner, .. } => match &**inner {
                        Type::Named(name) => name.clone(),
                        _ => return Err(SemanticError::UndefinedType(format!("{:?}", obj_type))),
                    },
                    _ => return Err(SemanticError::UndefinedType(format!("{:?}", obj_type))),
                };

                if let Some(symbol) = self.scopes.lookup(&type_name) {
                    if let SymbolKind::Struct { fields: _, methods } = &symbol.kind {
                        // Find method
                        for m in methods {
                            if &m.name == method {
                                if args.len() != m.params.len() {
                                    return Err(SemanticError::WrongNumberOfArguments {
                                        expected: m.params.len(),
                                        found: args.len(),
                                    });
                                }
                                return Ok(m.return_type.clone().unwrap_or(Type::Void));
                            }
                        }
                    }
                }

                Err(SemanticError::UndefinedFunction(method.clone()))
            }

            ExprKind::FieldAccess { object, field } => {
                let obj_type = self.check_expr(object)?;

                match &obj_type {
                    Type::Named(name) => {
                        if let Some(symbol) = self.scopes.lookup(name) {
                            if let SymbolKind::Struct { fields, .. } = &symbol.kind {
                                for (fname, ftype, _) in fields {
                                    if fname == field {
                                        return Ok(ftype.clone());
                                    }
                                }
                            }
                        }
                        Err(SemanticError::UndefinedVariable(field.clone()))
                    }
                    Type::Pointer { inner, .. } => {
                        if let Type::Named(name) = &**inner {
                            if let Some(symbol) = self.scopes.lookup(name) {
                                if let SymbolKind::Struct { fields, .. } = &symbol.kind {
                                    for (fname, ftype, _) in fields {
                                        if fname == field {
                                            return Ok(ftype.clone());
                                        }
                                    }
                                }
                            }
                        }
                        Err(SemanticError::UndefinedVariable(field.clone()))
                    }
                    _ => Err(SemanticError::UndefinedType(format!("{:?}", obj_type))),
                }
            }

            ExprKind::SafeAccess { object, field } => {
                let obj_type = self.check_expr(object)?;

                // Similar to FieldAccess but result is nullable
                match &obj_type {
                    Type::Named(name) => {
                        if let Some(symbol) = self.scopes.lookup(name) {
                            if let SymbolKind::Struct { fields, .. } = &symbol.kind {
                                for (fname, ftype, _) in fields {
                                    if fname == field {
                                        return Ok(Type::Nullable(Box::new(ftype.clone())));
                                    }
                                }
                            }
                        }
                        Err(SemanticError::UndefinedVariable(field.clone()))
                    }
                    Type::Pointer { inner, .. } => {
                        if let Type::Named(name) = &**inner {
                            if let Some(symbol) = self.scopes.lookup(name) {
                                if let SymbolKind::Struct { fields, .. } = &symbol.kind {
                                    for (fname, ftype, _) in fields {
                                        if fname == field {
                                            return Ok(Type::Nullable(Box::new(ftype.clone())));
                                        }
                                    }
                                }
                            }
                        }
                        Err(SemanticError::UndefinedVariable(field.clone()))
                    }
                    _ => Err(SemanticError::UndefinedType(format!("{:?}", obj_type))),
                }
            }

            ExprKind::Elvis { left, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                // Result is the inner type of left if left is nullable
                match left_type {
                    Type::Nullable(inner) => {
                        if self.types_compatible(&inner, &right_type) {
                            Ok(*inner)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: *inner,
                                found: right_type,
                            })
                        }
                    }
                    _ => Ok(left_type),
                }
            }

            ExprKind::ForceUnwrap(inner) => {
                let inner_type = self.check_expr(inner)?;

                match inner_type {
                    Type::Nullable(inner) => Ok(*inner),
                    _ => Err(SemanticError::NullSafetyViolation("not a nullable type".to_string())),
                }
            }

            ExprKind::Index { object, index } => {
                let obj_type = self.check_expr(object)?;
                let idx_type = self.check_expr(index)?;

                if idx_type != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Int,
                        found: idx_type,
                    });
                }

                match obj_type {
                    Type::Array(inner) => Ok(*inner),
                    Type::Generic { name, args } if name == "List" && !args.is_empty() => {
                        Ok(args[0].clone())
                    }
                    _ => Err(SemanticError::UndefinedType(format!("{:?}", obj_type))),
                }
            }

            ExprKind::StructInstance { name, fields: _, mutable: _ } => {
                self.check_type_exists(&Type::Named(name.clone()))?;
                Ok(Type::Named(name.clone()))
            }

            ExprKind::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Void)));
                }

                let elem_type = self.check_expr(&elements[0])?;
                for elem in &elements[1..] {
                    let t = self.check_expr(elem)?;
                    if !self.types_compatible(&elem_type, &t) {
                        return Err(SemanticError::TypeMismatch {
                            expected: elem_type.clone(),
                            found: t,
                        });
                    }
                }

                Ok(Type::Array(Box::new(elem_type)))
            }

            ExprKind::MapLiteral(entries) => {
                if entries.is_empty() {
                    return Ok(Type::Generic {
                        name: "Map".to_string(),
                        args: vec![Type::Void, Type::Void],
                    });
                }

                let key_type = self.check_expr(&entries[0].0)?;
                let value_type = self.check_expr(&entries[0].1)?;

                Ok(Type::Generic {
                    name: "Map".to_string(),
                    args: vec![key_type, value_type],
                })
            }

            ExprKind::Lambda { params, return_type, body } => {
                self.scopes.enter_scope();

                let mut param_types = Vec::new();
                for param in params {
                    let ty = param.type_annotation.clone().unwrap_or(Type::Void);
                    param_types.push(ty.clone());

                    let symbol = Symbol::variable(
                        param.name.clone(),
                        false,
                        ty,
                        self.scopes.current_level(),
                    );
                    self.scopes.define(&param.name, symbol);
                }

                match body {
                    LambdaBody::Expr(e) => {
                        let ret = self.check_expr(e)?;
                        self.scopes.exit_scope();

                        Ok(Type::Function {
                            params: param_types,
                            return_type: Box::new(return_type.clone().unwrap_or(ret)),
                        })
                    }
                    LambdaBody::Block(stmts) => {
                        for stmt in stmts {
                            self.check_stmt(stmt)?;
                        }
                        self.scopes.exit_scope();

                        Ok(Type::Function {
                            params: param_types,
                            return_type: Box::new(return_type.clone().unwrap_or(Type::Void)),
                        })
                    }
                }
            }

            ExprKind::If { condition, then_branch, else_branch } => {
                let cond_type = self.check_expr(condition)?;
                if cond_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: cond_type,
                    });
                }

                let then_type = self.check_expr(then_branch)?;

                if let Some(else_branch) = else_branch {
                    let else_type = self.check_expr(else_branch)?;
                    if !self.types_compatible(&then_type, &else_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: then_type,
                            found: else_type,
                        });
                    }
                }

                Ok(then_type)
            }

            ExprKind::Conditional { condition, then_expr, else_expr } => {
                let cond_type = self.check_expr(condition)?;
                if cond_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: cond_type,
                    });
                }

                let then_type = self.check_expr(then_expr)?;
                let else_type = self.check_expr(else_expr)?;

                if !self.types_compatible(&then_type, &else_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: then_type,
                        found: else_type,
                    });
                }

                Ok(then_type)
            }

            ExprKind::Assignment { target, value } => {
                // Check that target is assignable
                match &target.kind {
                    ExprKind::Ident(name) => {
                        let symbol = self.scopes.lookup(name).ok_or_else(|| {
                            SemanticError::UndefinedVariable(name.clone())
                        })?;

                        if !symbol.is_mutable() {
                            return Err(SemanticError::CannotAssignImmutable(name.clone()));
                        }
                    }
                    ExprKind::FieldAccess { .. } | ExprKind::Index { .. } => {}
                    _ => return Err(SemanticError::InvalidAssignmentTarget),
                }

                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;

                if !self.types_compatible(&target_type, &value_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: target_type,
                        found: value_type,
                    });
                }

                Ok(target_type)
            }

            ExprKind::Move(inner) => {
                let inner_type = self.check_expr(inner)?;

                // Mark variable as moved
                if let ExprKind::Ident(name) = &inner.kind {
                    if let Some(symbol) = self.scopes.lookup_mut(name) {
                        symbol.mark_moved();
                    }
                }

                Ok(inner_type)
            }

            ExprKind::Cast { expr, target_type } => {
                let _expr_type = self.check_expr(expr)?;
                self.check_type_exists(target_type)?;

                // For MVP, allow all casts (runtime will handle)
                Ok(target_type.clone())
            }

            ExprKind::TemplateLiteral(parts) => {
                self.check_template_literal(parts)
            }
        }
    }

    fn check_type_exists(&self, ty: &Type) -> Result<(), SemanticError> {
        match ty {
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Void => Ok(()),
            Type::Named(name) => {
                if self.scopes.lookup(name).is_none() {
                    Err(SemanticError::UndefinedType(name.clone()))
                } else {
                    Ok(())
                }
            }
            Type::Pointer { inner, .. } => self.check_type_exists(inner),
            Type::Nullable(inner) => self.check_type_exists(inner),
            Type::Array(inner) => self.check_type_exists(inner),
            Type::Generic { name, args } => {
                for arg in args {
                    self.check_type_exists(arg)?;
                }
                if self.scopes.lookup(name).is_none() {
                    Err(SemanticError::UndefinedType(name.clone()))
                } else {
                    Ok(())
                }
            }
            Type::Function { params, return_type } => {
                for param in params {
                    self.check_type_exists(param)?;
                }
                self.check_type_exists(return_type)
            }
        }
    }

    fn types_compatible(&self, expected: &Type, found: &Type) -> bool {
        match (expected, found) {
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            (Type::Void, Type::Void) => true,
            (Type::Named(a), Type::Named(b)) => a == b,
            (Type::Nullable(inner), found) => {
                self.types_compatible(inner, found) || matches!(found, Type::Nullable(_))
            }
            (found, Type::Nullable(inner)) => self.types_compatible(found, inner),
            (Type::Pointer { inner: a, mutable: ma }, Type::Pointer { inner: b, mutable: mb }) => {
                (*ma || !mb) && self.types_compatible(a, b)
            }
            (Type::Array(a), Type::Array(b)) => self.types_compatible(a, b),
            (Type::Generic { name: n1, args: a1 }, Type::Generic { name: n2, args: a2 }) => {
                n1 == n2 && a1.len() == a2.len() && a1.iter().zip(a2).all(|(a, b)| self.types_compatible(a, b))
            }
            _ => false,
        }
    }

    fn check_template_literal(&mut self, parts: &[TemplatePart]) -> Result<Type, SemanticError> {
        for part in parts {
            if let TemplatePart::Expr(expr) = part {
                let ty = self.check_expr(expr)?;
                if !self.is_stringifiable(&ty) {
                    return Err(SemanticError::CannotConvertToString {
                        ty,
                        span: expr.span,
                    });
                }
            }
        }
        Ok(Type::String)
    }

    fn is_stringifiable(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float | Type::Bool | Type::String)
    }

    /// Parse printf format string and return expected types
    fn parse_printf_format(&self, format: &str) -> Result<Vec<Type>, SemanticError> {
        let mut types = Vec::new();
        let chars: Vec<char> = format.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '%' {
                i += 1;
                if i >= chars.len() {
                    // Trailing % - not an error, just output %
                    break;
                }

                // Skip width/precision modifiers
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '-') {
                    i += 1;
                }

                if i >= chars.len() {
                    break;
                }

                match chars[i] {
                    '%' => {} // Escaped %, no argument
                    'd' | 'i' | 'x' | 'X' | 'o' | 'c' | 'l' => {
                        types.push(Type::Int);
                    }
                    'f' => {
                        types.push(Type::Float);
                    }
                    's' => {
                        types.push(Type::String);
                    }
                    'b' => {
                        types.push(Type::Bool);
                    }
                    unknown => {
                        return Err(SemanticError::InvalidFormatSpecifier(unknown));
                    }
                }
            }
            i += 1;
        }

        Ok(types)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}