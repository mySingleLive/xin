//! IR Builder

use xin_ast::{BinOp as AstBinOp, Decl, DeclKind, Expr, ExprKind, FuncDecl, SourceFile, Stmt, StmtKind, Type};

use crate::{BinOp, ConcatType, ExternFunction, Instruction, IRFunction, IRModule, IRType, Value};

/// IR Builder
pub struct IRBuilder {
    module: IRModule,
    current_function: Option<IRFunction>,
    temp_counter: usize,
    label_counter: usize,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            module: IRModule::new(),
            current_function: None,
            temp_counter: 0,
            label_counter: 0,
        }
    }

    pub fn build(&mut self, file: &SourceFile) -> IRModule {
        for decl in &file.declarations {
            self.build_declaration(decl);
        }
        self.module.clone()
    }

    fn build_declaration(&mut self, decl: &Decl) {
        match &decl.kind {
            DeclKind::Func(f) => self.build_function(f),
            DeclKind::Struct(_) | DeclKind::Interface(_) | DeclKind::Import(_) => {}
        }
    }

    fn build_function(&mut self, func: &FuncDecl) {
        let params: Vec<(String, IRType)> = func
            .params
            .iter()
            .map(|p| (p.name.clone(), self.convert_type(&p.type_annotation)))
            .collect();

        let return_type = func
            .return_type
            .as_ref()
            .map(|t| self.convert_type(t))
            .unwrap_or(IRType::Void);

        self.current_function = Some(IRFunction {
            name: func.name.clone(),
            params: params.clone(),
            return_type: return_type.clone(),
            instructions: Vec::new(),
        });

        // Allocate space for parameters
        for (name, ty) in &params {
            let ptr = Value(format!("%{}", name));
            self.emit(Instruction::Alloca {
                result: ptr.clone(),
                ty: ty.clone(),
            });
        }

        // Build body
        match &func.body {
            xin_ast::FuncBody::Block(stmts) => {
                for stmt in stmts {
                    self.build_stmt(stmt);
                }
            }
            xin_ast::FuncBody::Expr(expr) => {
                let value = self.build_expr(expr);
                if let Some(v) = value {
                    self.emit(Instruction::Return(Some(v)));
                } else {
                    self.emit(Instruction::Return(None));
                }
            }
        }

        // Add implicit return if needed
        if let Some(f) = &self.current_function {
            if let Some(last) = f.instructions.last() {
                if !matches!(last, Instruction::Return(_) | Instruction::Jump(_)) {
                    self.emit(Instruction::Return(None));
                }
            }
        }

        if let Some(f) = self.current_function.take() {
            self.module.add_function(f);
        }
    }

    fn build_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::VarDecl(var) => {
                if let Some(value) = &var.value {
                    let val = self.build_expr(value);
                    if let Some(v) = val {
                        let ptr = self.new_temp();
                        let ty = var.type_annotation
                            .as_ref()
                            .map(|t| self.convert_type(t))
                            .unwrap_or(IRType::I64);
                        self.emit(Instruction::Alloca {
                            result: ptr.clone(),
                            ty,
                        });
                        self.emit(Instruction::Store { value: v, ptr });
                    }
                }
            }
            StmtKind::Expr(expr) => {
                self.build_expr(expr);
            }
            StmtKind::Return(value) => {
                let val = value.as_ref().and_then(|e| self.build_expr(e));
                self.emit(Instruction::Return(val));
            }
            StmtKind::If { condition, then_block, else_block } => {
                let cond = self.build_expr(condition).unwrap();
                let then_label = self.new_label();
                let else_label = self.new_label();
                let end_label = self.new_label();

                self.emit(Instruction::Branch {
                    cond,
                    then_label: then_label.clone(),
                    else_label: else_label.clone(),
                });

                self.emit(Instruction::Label(then_label));
                for stmt in then_block {
                    self.build_stmt(stmt);
                }
                self.emit(Instruction::Jump(end_label.clone()));

                self.emit(Instruction::Label(else_label));
                if let Some(else_block) = else_block {
                    for stmt in else_block {
                        self.build_stmt(stmt);
                    }
                }
                self.emit(Instruction::Jump(end_label.clone()));

                self.emit(Instruction::Label(end_label));
            }
            StmtKind::For(for_loop) => {
                match for_loop {
                    xin_ast::ForLoop::CStyle { init, condition, update, body } => {
                        if let Some(init) = init {
                            self.build_stmt(init);
                        }

                        let cond_label = self.new_label();
                        let body_label = self.new_label();
                        let end_label = self.new_label();

                        self.emit(Instruction::Label(cond_label.clone()));

                        if let Some(cond) = condition {
                            let cond_val = self.build_expr(cond).unwrap();
                            self.emit(Instruction::Branch {
                                cond: cond_val,
                                then_label: body_label.clone(),
                                else_label: end_label.clone(),
                            });
                        }

                        self.emit(Instruction::Label(body_label));
                        for stmt in body {
                            self.build_stmt(stmt);
                        }
                        if let Some(update) = update {
                            self.build_expr(update);
                        }
                        self.emit(Instruction::Jump(cond_label));

                        self.emit(Instruction::Label(end_label));
                    }
                    xin_ast::ForLoop::ForIn { var_name: _, iterable, body } => {
                        // Simplified: just emit body
                        // TODO: Implement proper iteration
                        let _ = self.build_expr(iterable);
                        for stmt in body {
                            self.build_stmt(stmt);
                        }
                    }
                    xin_ast::ForLoop::While { condition, body } => {
                        let cond_label = self.new_label();
                        let body_label = self.new_label();
                        let end_label = self.new_label();

                        self.emit(Instruction::Label(cond_label.clone()));
                        let cond_val = self.build_expr(condition).unwrap();
                        self.emit(Instruction::Branch {
                            cond: cond_val,
                            then_label: body_label.clone(),
                            else_label: end_label.clone(),
                        });

                        self.emit(Instruction::Label(body_label));
                        for stmt in body {
                            self.build_stmt(stmt);
                        }
                        self.emit(Instruction::Jump(cond_label));

                        self.emit(Instruction::Label(end_label));
                    }
                    xin_ast::ForLoop::Infinite { body } => {
                        let body_label = self.new_label();
                        self.emit(Instruction::Label(body_label.clone()));
                        for stmt in body {
                            self.build_stmt(stmt);
                        }
                        self.emit(Instruction::Jump(body_label));
                    }
                }
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Block(stmts) => {
                for stmt in stmts {
                    self.build_stmt(stmt);
                }
            }
        }
    }

    fn build_expr(&mut self, expr: &Expr) -> Option<Value> {
        match &expr.kind {
            ExprKind::IntLiteral(n) => {
                let result = self.new_temp();
                self.emit(Instruction::Const {
                    result: result.clone(),
                    value: n.to_string(),
                    ty: IRType::I64,
                });
                Some(result)
            }
            ExprKind::FloatLiteral(n) => {
                let result = self.new_temp();
                self.emit(Instruction::Const {
                    result: result.clone(),
                    value: n.to_string(),
                    ty: IRType::F64,
                });
                Some(result)
            }
            ExprKind::StringLiteral(s) => {
                // Add string to module's string table and use StringConst
                let string_index = self.module.add_string(s);
                let result = self.new_temp();
                self.emit(Instruction::StringConst {
                    result: result.clone(),
                    string_index,
                });
                Some(result)
            }
            ExprKind::BoolLiteral(b) => {
                let result = self.new_temp();
                self.emit(Instruction::Const {
                    result: result.clone(),
                    value: b.to_string(),
                    ty: IRType::Bool,
                });
                Some(result)
            }
            ExprKind::Null => None,
            ExprKind::Ident(name) => {
                // For now, just load the variable
                let ptr = Value(format!("%{}", name));
                let result = self.new_temp();
                self.emit(Instruction::Load {
                    result: result.clone(),
                    ptr,
                });
                Some(result)
            }
            ExprKind::Binary { op, left, right } => {
                let left_val = self.build_expr(left)?;
                let right_val = self.build_expr(right)?;

                // Check if this is string concatenation
                let left_type = Self::get_expr_type(left);
                let right_type = Self::get_expr_type(right);

                if *op == AstBinOp::Add {
                    let is_string_concat = matches!(left_type, Some(Type::String))
                        || matches!(right_type, Some(Type::String));

                    if is_string_concat {
                        let left_concat_type = self.type_to_concat_type(&left_type);
                        let right_concat_type = self.type_to_concat_type(&right_type);

                        let result = self.new_temp();
                        self.emit(Instruction::StringConcat {
                            result: result.clone(),
                            left: left_val,
                            left_type: left_concat_type,
                            right: right_val,
                            right_type: right_concat_type,
                        });

                        // Declare the external concat function
                        self.declare_str_concat_extern(left_concat_type, right_concat_type);

                        return Some(result);
                    }
                }

                // Regular binary operation
                let result = self.new_temp();
                self.emit(Instruction::Binary {
                    result: result.clone(),
                    op: self.convert_binop(op),
                    left: left_val,
                    right: right_val,
                });
                Some(result)
            }
            ExprKind::Unary { op, operand } => {
                let operand_val = self.build_expr(operand)?;
                let result = self.new_temp();
                match op {
                    xin_ast::UnaryOp::Neg => {
                        let zero = self.new_temp();
                        self.emit(Instruction::Const {
                            result: zero.clone(),
                            value: "0".to_string(),
                            ty: IRType::I64,
                        });
                        self.emit(Instruction::Binary {
                            result: result.clone(),
                            op: BinOp::Sub,
                            left: zero,
                            right: operand_val,
                        });
                    }
                    xin_ast::UnaryOp::Not => {
                        let one = self.new_temp();
                        self.emit(Instruction::Const {
                            result: one.clone(),
                            value: "1".to_string(),
                            ty: IRType::Bool,
                        });
                        self.emit(Instruction::Binary {
                            result: result.clone(),
                            op: BinOp::Eq,
                            left: operand_val,
                            right: one,
                        });
                    }
                }
                Some(result)
            }
            ExprKind::Call { callee, args } => {
                match &callee.kind {
                    ExprKind::Ident(name) => {
                        // Handle println/print specially
                        if name == "println" {
                            return self.handle_println(args);
                        } else if name == "print" {
                            return self.handle_print(args);
                        }

                        // Regular function call
                        let arg_vals: Vec<Value> = args.iter().filter_map(|a| self.build_expr(a)).collect();
                        let result = self.new_temp();
                        self.emit(Instruction::Call {
                            result: Some(result.clone()),
                            func: name.clone(),
                            args: arg_vals,
                            is_extern: false,
                        });
                        Some(result)
                    }
                    _ => None,
                }
            }
            ExprKind::MethodCall { object, method, args } => {
                let _obj_val = self.build_expr(object)?;
                let arg_vals: Vec<Value> = args.iter().filter_map(|a| self.build_expr(a)).collect();

                let result = self.new_temp();
                // Method call as function call with self parameter
                self.emit(Instruction::Call {
                    result: Some(result.clone()),
                    func: method.clone(),
                    args: arg_vals,
                    is_extern: false,
                });
                Some(result)
            }
            ExprKind::Assignment { target, value } => {
                let val = self.build_expr(value)?;
                match &target.kind {
                    ExprKind::Ident(name) => {
                        let ptr = Value(format!("%{}", name));
                        self.emit(Instruction::Store { value: val.clone(), ptr });
                    }
                    _ => {}
                }
                Some(val)
            }
            ExprKind::Conditional { condition, then_expr, else_expr } => {
                let cond = self.build_expr(condition)?;
                let result = self.new_temp();

                let then_label = self.new_label();
                let else_label = self.new_label();
                let end_label = self.new_label();

                self.emit(Instruction::Branch {
                    cond,
                    then_label: then_label.clone(),
                    else_label: else_label.clone(),
                });

                self.emit(Instruction::Label(then_label.clone()));
                let then_val = self.build_expr(then_expr)?;
                self.emit(Instruction::Jump(end_label.clone()));

                self.emit(Instruction::Label(else_label.clone()));
                let else_val = self.build_expr(else_expr)?;
                self.emit(Instruction::Jump(end_label.clone()));

                self.emit(Instruction::Label(end_label));
                self.emit(Instruction::Phi {
                    result: result.clone(),
                    incoming: vec![
                        (then_val, then_label),
                        (else_val, else_label),
                    ],
                });

                Some(result)
            }
            ExprKind::Move(inner) => self.build_expr(inner),
            ExprKind::Cast { expr, target_type: _ } => self.build_expr(expr),
            _ => None,
        }
    }

    fn convert_type(&self, ty: &Type) -> IRType {
        match ty {
            Type::Int => IRType::I64,
            Type::Float => IRType::F64,
            Type::Bool => IRType::Bool,
            Type::String => IRType::String,
            Type::Void => IRType::Void,
            Type::Named(name) => IRType::Ptr(name.clone()),
            Type::Pointer { inner, .. } => match &**inner {
                Type::Named(name) => IRType::Ptr(name.clone()),
                _ => IRType::Ptr("unknown".to_string()),
            },
            _ => IRType::I64,
        }
    }

    fn convert_binop(&self, op: &AstBinOp) -> BinOp {
        match op {
            AstBinOp::Add => BinOp::Add,
            AstBinOp::Sub => BinOp::Sub,
            AstBinOp::Mul => BinOp::Mul,
            AstBinOp::Div => BinOp::Div,
            AstBinOp::Mod => BinOp::Mod,
            AstBinOp::Eq => BinOp::Eq,
            AstBinOp::Ne => BinOp::Ne,
            AstBinOp::Lt => BinOp::Lt,
            AstBinOp::Gt => BinOp::Gt,
            AstBinOp::Le => BinOp::Le,
            AstBinOp::Ge => BinOp::Ge,
            AstBinOp::And => BinOp::And,
            AstBinOp::Or => BinOp::Or,
        }
    }

    fn new_temp(&mut self) -> Value {
        self.temp_counter += 1;
        Value(format!("%t{}", self.temp_counter))
    }

    fn new_label(&mut self) -> String {
        self.label_counter += 1;
        format!("L{}", self.label_counter)
    }

    fn emit(&mut self, instr: Instruction) {
        if let Some(f) = &mut self.current_function {
            f.instructions.push(instr);
        }
    }

    /// Handle println(expr) - prints value followed by newline
    fn handle_println(&mut self, args: &[Expr]) -> Option<Value> {
        if args.is_empty() {
            // println() with no args - just print newline
            self.emit(Instruction::Call {
                result: None,
                func: "xin_println".to_string(),
                args: vec![],
                is_extern: true,
            });
            // Declare external function if not already declared
            self.declare_extern_if_needed("xin_println", vec![], None);
            return None;
        }

        let arg = &args[0];
        let arg_val = self.build_expr(arg)?;
        let arg_type = Self::get_expr_type(arg);

        match arg_type {
            Some(Type::Int) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_int".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_println".to_string(),
                    args: vec![],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_int", vec![IRType::I64], None);
                self.declare_extern_if_needed("xin_println", vec![], None);
            }
            Some(Type::Float) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_float".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_println".to_string(),
                    args: vec![],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_float", vec![IRType::F64], None);
                self.declare_extern_if_needed("xin_println", vec![], None);
            }
            Some(Type::Bool) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_bool".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_println".to_string(),
                    args: vec![],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_bool", vec![IRType::Bool], None);
                self.declare_extern_if_needed("xin_println", vec![], None);
            }
            Some(Type::String) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_str".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_println".to_string(),
                    args: vec![],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_str", vec![IRType::Ptr("char".to_string())], None);
                self.declare_extern_if_needed("xin_println", vec![], None);
            }
            _ => {
                // Default to int
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_int".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_println".to_string(),
                    args: vec![],
                    is_extern: true,
                });
            }
        }
        None
    }

    /// Handle print(expr) - prints value without newline
    fn handle_print(&mut self, args: &[Expr]) -> Option<Value> {
        if args.is_empty() {
            return None;
        }

        let arg = &args[0];
        let arg_val = self.build_expr(arg)?;
        let arg_type = Self::get_expr_type(arg);

        match arg_type {
            Some(Type::Int) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_int".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_int", vec![IRType::I64], None);
            }
            Some(Type::Float) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_float".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_float", vec![IRType::F64], None);
            }
            Some(Type::Bool) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_bool".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_bool", vec![IRType::Bool], None);
            }
            Some(Type::String) => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_str".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
                self.declare_extern_if_needed("xin_print_str", vec![IRType::Ptr("char".to_string())], None);
            }
            _ => {
                self.emit(Instruction::Call {
                    result: None,
                    func: "xin_print_int".to_string(),
                    args: vec![arg_val],
                    is_extern: true,
                });
            }
        }
        None
    }

    /// Declare an external function if not already declared
    fn declare_extern_if_needed(&mut self, name: &str, params: Vec<IRType>, return_type: Option<IRType>) {
        // Check if already declared
        if self.module.extern_functions.iter().any(|f| f.name == name) {
            return;
        }
        self.module.add_extern_function(ExternFunction {
            name: name.to_string(),
            params,
            return_type,
        });
    }

    /// Get the type of an expression (simplified)
    fn get_expr_type(expr: &Expr) -> Option<Type> {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Some(Type::Int),
            ExprKind::FloatLiteral(_) => Some(Type::Float),
            ExprKind::BoolLiteral(_) => Some(Type::Bool),
            ExprKind::StringLiteral(_) => Some(Type::String),
            ExprKind::Ident(_) => None, // Would need symbol table
            _ => None,
        }
    }

    /// Convert Type to ConcatType
    fn type_to_concat_type(&self, ty: &Option<Type>) -> ConcatType {
        match ty {
            Some(Type::Int) => ConcatType::Int,
            Some(Type::Float) => ConcatType::Float,
            Some(Type::Bool) => ConcatType::Bool,
            Some(Type::String) | None => ConcatType::String,
            _ => ConcatType::String,
        }
    }

    /// Declare string concat extern function if needed
    fn declare_str_concat_extern(&mut self, left_type: ConcatType, right_type: ConcatType) {
        let func_name = match (left_type, right_type) {
            (ConcatType::String, ConcatType::String) => "xin_str_concat_ss",
            (ConcatType::String, ConcatType::Int) => "xin_str_concat_si",
            (ConcatType::Int, ConcatType::String) => "xin_str_concat_is",
            (ConcatType::String, ConcatType::Float) => "xin_str_concat_sf",
            (ConcatType::Float, ConcatType::String) => "xin_str_concat_fs",
            (ConcatType::String, ConcatType::Bool) => "xin_str_concat_sb",
            (ConcatType::Bool, ConcatType::String) => "xin_str_concat_bs",
            _ => "xin_str_concat_ss", // fallback
        };

        let param_types = match (left_type, right_type) {
            (ConcatType::String, ConcatType::String) => vec![IRType::Ptr("char".to_string()), IRType::Ptr("char".to_string())],
            (ConcatType::String, ConcatType::Int) => vec![IRType::Ptr("char".to_string()), IRType::I64],
            (ConcatType::Int, ConcatType::String) => vec![IRType::I64, IRType::Ptr("char".to_string())],
            (ConcatType::String, ConcatType::Float) => vec![IRType::Ptr("char".to_string()), IRType::F64],
            (ConcatType::Float, ConcatType::String) => vec![IRType::F64, IRType::Ptr("char".to_string())],
            (ConcatType::String, ConcatType::Bool) => vec![IRType::Ptr("char".to_string()), IRType::Bool],
            (ConcatType::Bool, ConcatType::String) => vec![IRType::Bool, IRType::Ptr("char".to_string())],
            _ => vec![IRType::Ptr("char".to_string()), IRType::Ptr("char".to_string())],
        };

        self.declare_extern_if_needed(func_name, param_types, Some(IRType::Ptr("char".to_string())));
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}