//! AOT (Ahead-of-Time) code generator using Cranelift ObjectModule

use cranelift::prelude::*;
use cranelift::codegen::ir::{ExtFuncData, ExternalName, UserExternalName, FuncRef, GlobalValue};
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;
use xin_ir::{BinOp, ExternFunction, Instruction, IRFunction, IRModule, IRType};

/// AOT Code generator using Cranelift ObjectModule
pub struct AOTCodeGenerator {
    module: ObjectModule,
    pointer_type: Type,
    /// Cache of external function IDs
    extern_func_ids: std::collections::HashMap<String, FuncId>,
    /// Cache of string data IDs
    string_data_ids: std::collections::HashMap<usize, DataId>,
    /// All function signatures (both external and internal)
    func_sigs: std::collections::HashMap<String, cranelift::codegen::ir::Signature>,
}

impl AOTCodeGenerator {
    /// Create a new AOT code generator
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "true").unwrap(); // Enable PIC to avoid alignment issues with embedded addresses

        let isa_builder = cranelift_native::builder()
            .map_err(|e| format!("Failed to create ISA builder: {}", e))?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| format!("Failed to create ISA: {}", e))?;

        let builder = cranelift_object::ObjectBuilder::new(
            isa,
            "xin_module",
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| format!("Failed to create object builder: {}", e))?;

        let module = ObjectModule::new(builder);
        let pointer_type = module.target_config().pointer_type();

        Ok(Self {
            module,
            pointer_type,
            extern_func_ids: std::collections::HashMap::new(),
            string_data_ids: std::collections::HashMap::new(),
            func_sigs: std::collections::HashMap::new(),
        })
    }

    /// Compile an IR module
    pub fn compile(&mut self, module: &IRModule) -> Result<(), String> {
        // First, declare all external functions
        for extern_func in &module.extern_functions {
            self.declare_extern_function(extern_func)?;
        }

        // Then, declare all string constants
        for (i, s) in module.strings.iter().enumerate() {
            self.declare_string_constant(i, s)?;
        }

        // Then, forward-declare all internal functions
        for func in &module.functions {
            self.declare_function(func)?;
        }

        // Finally compile all functions
        for func in &module.functions {
            self.compile_function(func)?;
        }

        Ok(())
    }

    /// Declare an external function
    fn declare_extern_function(&mut self, func: &ExternFunction) -> Result<(), String> {
        let mut sig = self.module.make_signature();
        for ty in &func.params {
            sig.params.push(AbiParam::new(self.convert_type(ty)));
        }
        if let Some(ret_ty) = &func.return_type {
            sig.returns.push(AbiParam::new(self.convert_type(ret_ty)));
        }

        let func_id = self
            .module
            .declare_function(&func.name, Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare external function: {}", e))?;

        self.extern_func_ids.insert(func.name.clone(), func_id);
        // Store the signature for later use when calling this function
        self.func_sigs.insert(func.name.clone(), sig);

        Ok(())
    }

    /// Declare a string constant in the data section
    fn declare_string_constant(&mut self, index: usize, s: &str) -> Result<(), String> {
        // Create a null-terminated string
        let mut data: Vec<u8> = s.bytes().collect();
        data.push(0); // null terminator

        let name = format!("__str_{}", index);
        let data_id = self
            .module
            .declare_data(&name, Linkage::Local, false, false)
            .map_err(|e| format!("Failed to declare string data: {}", e))?;

        let mut data_desc = DataDescription::new();
        data_desc.define(data.into_boxed_slice());
        // Set 8-byte alignment for proper pointer alignment
        data_desc.align = Some(8);

        self.module
            .define_data(data_id, &data_desc)
            .map_err(|e| format!("Failed to define string data: {}", e))?;

        self.string_data_ids.insert(index, data_id);

        Ok(())
    }

    /// Declare an internal function (forward declaration)
    fn declare_function(&mut self, func: &IRFunction) -> Result<(), String> {
        let mut sig = self.module.make_signature();
        for (_, ty) in &func.params {
            sig.params.push(AbiParam::new(self.convert_type(ty)));
        }
        if func.return_type != IRType::Void {
            sig.returns.push(AbiParam::new(self.convert_type(
                &func.return_type,
            )));
        }

        let func_id = self
            .module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Failed to declare function: {}", e))?;

        // Store the signature for later use when calling this function
        self.func_sigs.insert(func.name.clone(), sig);

        Ok(())
    }

    /// Compile a single function
    fn compile_function(&mut self, func: &IRFunction) -> Result<(), String> {
        // Get the function ID from the module
        let func_id = self
            .module
            .get_name(&func.name)
            .ok_or_else(|| format!("Function {} not found in module", func.name))?;
        let func_id = match func_id {
            cranelift_module::FuncOrDataId::Func(id) => id,
            _ => return Err(format!("{} is not a function", func.name)),
        };

        // Get the signature we stored earlier
        let sig = self.func_sigs.get(&func.name)
            .ok_or_else(|| format!("Signature not found for function {}", func.name))?
            .clone();

        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;

        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Variable tracking
        let mut variables: std::collections::HashMap<String, Variable> =
            std::collections::HashMap::new();
        let mut var_counter = 0;

        // Process parameters
        for (name, _) in &func.params {
            let var = Variable::new(var_counter);
            var_counter += 1;
            variables.insert(name.clone(), var);
        }

        // Cache for function refs and global values
        let mut func_ref_cache: std::collections::HashMap<String, FuncRef> =
            std::collections::HashMap::new();
        let mut global_value_cache: std::collections::HashMap<usize, GlobalValue> =
            std::collections::HashMap::new();

        // Process instructions
        for instr in &func.instructions {
            self.compile_instruction(
                &mut builder,
                instr,
                &mut variables,
                &mut var_counter,
                &mut func_ref_cache,
                &mut global_value_cache,
            )?;
        }

        builder.finalize();

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| {
                format!("Failed to define function: {}", e)
            })?;

        self.module.clear_context(&mut ctx);

        Ok(())
    }

    /// Compile a single instruction
    fn compile_instruction(
        &self,
        builder: &mut FunctionBuilder,
        instr: &Instruction,
        variables: &mut std::collections::HashMap<String, Variable>,
        var_counter: &mut usize,
        func_ref_cache: &mut std::collections::HashMap<String, FuncRef>,
        global_value_cache: &mut std::collections::HashMap<usize, GlobalValue>,
    ) -> Result<(), String> {
        match instr {
            Instruction::Const { result, value, ty } => {
                let val = match ty {
                    IRType::I64 => {
                        let n: i64 = value.parse().unwrap_or(0);
                        builder.ins().iconst(types::I64, n)
                    }
                    IRType::F64 => {
                        let n: f64 = value.parse().unwrap_or(0.0);
                        builder.ins().f64const(n)
                    }
                    IRType::Bool => {
                        let b = value == "true";
                        builder.ins().iconst(types::I8, i64::from(b))
                    }
                    _ => builder.ins().iconst(types::I64, 0),
                };
                self.store_variable(builder, result, val, variables, var_counter, types::I64);
            }
            Instruction::StringConst { result, string_index } => {
                let global_value = if let Some(gv) = global_value_cache.get(string_index) {
                    *gv
                } else {
                    // Get the DataId for this string
                    let data_id = self.string_data_ids.get(string_index)
                        .ok_or_else(|| format!("String index {} not found in data IDs", string_index))?;

                    // Use the module's declare_data_in_func to get the correct GlobalValue
                    let gv = self.module.declare_data_in_func(*data_id, builder.func);
                    global_value_cache.insert(*string_index, gv);
                    gv
                };
                let addr = builder.ins().global_value(self.pointer_type, global_value);
                self.store_variable(builder, result, addr, variables, var_counter, self.pointer_type);
            }
            Instruction::Binary { result, op, left, right } => {
                let left_val = self.load_variable(builder, left, variables)?;
                let right_val = self.load_variable(builder, right, variables)?;

                let val = match op {
                    BinOp::Add => builder.ins().iadd(left_val, right_val),
                    BinOp::Sub => builder.ins().isub(left_val, right_val),
                    BinOp::Mul => builder.ins().imul(left_val, right_val),
                    BinOp::Div => builder.ins().sdiv(left_val, right_val),
                    BinOp::Mod => builder.ins().srem(left_val, right_val),
                    BinOp::Eq => {
                        let cmp = builder.ins().icmp(IntCC::Equal, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::Ne => {
                        let cmp = builder.ins().icmp(IntCC::NotEqual, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::Lt => {
                        let cmp = builder.ins().icmp(IntCC::SignedLessThan, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::Gt => {
                        let cmp =
                            builder.ins().icmp(IntCC::SignedGreaterThan, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::Le => {
                        let cmp = builder
                            .ins()
                            .icmp(IntCC::SignedLessThanOrEqual, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::Ge => {
                        let cmp = builder
                            .ins()
                            .icmp(IntCC::SignedGreaterThanOrEqual, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::And => builder.ins().band(left_val, right_val),
                    BinOp::Or => builder.ins().bor(left_val, right_val),
                };
                self.store_variable(builder, result, val, variables, var_counter, types::I64);
            }
            Instruction::Return(val) => {
                if let Some(v) = val {
                    let ret_val = self.load_variable(builder, v, variables)?;
                    builder.ins().return_(&[ret_val]);
                } else {
                    builder.ins().return_(&[]);
                }
            }
            Instruction::Call { result, func: func_name, args, is_extern: _ } => {
                let func_ref = if let Some(fr) = func_ref_cache.get(func_name) {
                    *fr
                } else {
                    // Get function ID
                    let func_id = if let Some(id) = self.extern_func_ids.get(func_name) {
                        *id
                    } else {
                        let id = self
                            .module
                            .get_name(func_name)
                            .ok_or_else(|| format!("Function {} not found", func_name))?;
                        match id {
                            cranelift_module::FuncOrDataId::Func(id) => id,
                            _ => return Err(format!("{} is not a function", func_name)),
                        }
                    };

                    // Get the correct signature for this function
                    let sig = self.func_sigs.get(func_name)
                        .ok_or_else(|| format!("Signature not found for function {}", func_name))?
                        .clone();
                    let sig_ref = builder.func.import_signature(sig);

                    // Declare the function reference
                    let user_func_name = builder.func.declare_imported_user_function(UserExternalName {
                        namespace: 0,
                        index: func_id.as_u32(),
                    });

                    let fr = builder.import_function(ExtFuncData {
                        name: ExternalName::user(user_func_name),
                        signature: sig_ref,
                        colocated: true,
                    });
                    func_ref_cache.insert(func_name.clone(), fr);
                    fr
                };

                // Load arguments
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.load_variable(builder, a, variables))
                    .collect::<Result<Vec<_>, _>>()?;

                // Make the call
                let call_val = builder.ins().call(func_ref, &arg_vals);

                // Store result if any
                if let Some(result) = result {
                    // Get the return value (first return value)
                    let ret_val = builder.inst_results(call_val)[0];
                    self.store_variable(builder, result, ret_val, variables, var_counter, types::I64);
                }
            }
            Instruction::Jump(_) | Instruction::Branch { .. } | Instruction::Label(_) => {
                // TODO: Implement control flow
            }
            Instruction::Alloca { .. } | Instruction::Store { .. } | Instruction::Load { .. } => {
                // TODO: Implement memory operations
            }
            Instruction::Phi { .. } => {
                // TODO: Implement phi nodes
            }
            Instruction::StringConcat { .. } => {
                // TODO: Implement string concatenation (Task 8)
            }
            Instruction::StringFree { .. } => {
                // TODO: Implement string deallocation (Task 8)
            }
        }
        Ok(())
    }

    fn load_variable(
        &self,
        builder: &mut FunctionBuilder,
        value: &xin_ir::Value,
        variables: &std::collections::HashMap<String, Variable>,
    ) -> Result<Value, String> {
        let name = &value.0;
        if let Some(var) = variables.get(name) {
            Ok(builder.use_var(*var))
        } else {
            // Return a default value for undefined variables
            Ok(builder.ins().iconst(types::I64, 0))
        }
    }

    fn store_variable(
        &self,
        builder: &mut FunctionBuilder,
        result: &xin_ir::Value,
        value: Value,
        variables: &mut std::collections::HashMap<String, Variable>,
        var_counter: &mut usize,
        ty: Type,
    ) {
        let var = Variable::new(*var_counter);
        *var_counter += 1;
        variables.insert(result.0.clone(), var);
        builder.declare_var(var, ty);
        builder.def_var(var, value);
    }

    fn convert_type(&self, ty: &IRType) -> Type {
        match ty {
            IRType::I64 => types::I64,
            IRType::F64 => types::F64,
            IRType::Bool => types::I8,
            IRType::String => self.pointer_type,
            IRType::Void => panic!("Void type should not be converted to Cranelift type"),
            IRType::Ptr(_) => self.pointer_type,
        }
    }

    /// Emit the compiled object file
    pub fn emit_object(self) -> Result<Vec<u8>, String> {
        self.module
            .finish()
            .emit()
            .map_err(|e| format!("Failed to emit object file: {}", e))
    }
}

impl Default for AOTCodeGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create AOT code generator")
    }
}