//! AOT (Ahead-of-Time) code generator using Cranelift ObjectModule

use cranelift::prelude::*;
use cranelift::codegen::ir::{ExtFuncData, ExternalName, UserExternalName, FuncRef, GlobalValue};
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;
use xin_ir::{BinOp, ConcatType, ExternFunction, Instruction, IRFunction, IRModule, IRType};

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
        // First, declare array runtime functions
        self.declare_array_runtime_functions()?;

        // Then, declare all external functions from IR
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

    /// Declare array runtime functions
    fn declare_array_runtime_functions(&mut self) -> Result<(), String> {
        // xin_array_new(int64_t capacity) -> xin_array*
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(self.pointer_type));
        let func_id = self.module
            .declare_function("xin_array_new", Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare xin_array_new: {}", e))?;
        self.extern_func_ids.insert("xin_array_new".to_string(), func_id);
        self.func_sigs.insert("xin_array_new".to_string(), sig);

        // xin_array_get(xin_array* arr, int64_t index) -> void*
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(self.pointer_type));
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(self.pointer_type));
        let func_id = self.module
            .declare_function("xin_array_get", Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare xin_array_get: {}", e))?;
        self.extern_func_ids.insert("xin_array_get".to_string(), func_id);
        self.func_sigs.insert("xin_array_get".to_string(), sig);

        // xin_array_set(xin_array* arr, int64_t index, void* value) -> void
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(self.pointer_type));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(self.pointer_type));
        let func_id = self.module
            .declare_function("xin_array_set", Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare xin_array_set: {}", e))?;
        self.extern_func_ids.insert("xin_array_set".to_string(), func_id);
        self.func_sigs.insert("xin_array_set".to_string(), sig);

        // xin_array_push(xin_array* arr, void* value) -> void
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(self.pointer_type));
        sig.params.push(AbiParam::new(self.pointer_type));
        let func_id = self.module
            .declare_function("xin_array_push", Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare xin_array_push: {}", e))?;
        self.extern_func_ids.insert("xin_array_push".to_string(), func_id);
        self.func_sigs.insert("xin_array_push".to_string(), sig);

        // xin_array_pop(xin_array* arr) -> void*
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(self.pointer_type));
        sig.returns.push(AbiParam::new(self.pointer_type));
        let func_id = self.module
            .declare_function("xin_array_pop", Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare xin_array_pop: {}", e))?;
        self.extern_func_ids.insert("xin_array_pop".to_string(), func_id);
        self.func_sigs.insert("xin_array_pop".to_string(), sig);

        // xin_array_len(xin_array* arr) -> int64_t
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(self.pointer_type));
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = self.module
            .declare_function("xin_array_len", Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare xin_array_len: {}", e))?;
        self.extern_func_ids.insert("xin_array_len".to_string(), func_id);
        self.func_sigs.insert("xin_array_len".to_string(), sig);

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

        // Process parameters - bind IR parameter values to Cranelift block params
        for (i, (name, ty)) in func.params.iter().enumerate() {
            // The IR uses %param_N for the incoming parameter value
            let param_val_name = format!("%param_{}", i);
            let param_var = Variable::new(var_counter);
            var_counter += 1;
            variables.insert(param_val_name.clone(), param_var);

            // Get the block parameter value from Cranelift
            let cranelift_ty = self.convert_type(ty);
            let block_param = builder.block_params(entry_block)[i];
            builder.declare_var(param_var, cranelift_ty);
            builder.def_var(param_var, block_param);

            // Also create a variable for the parameter name (used for variable lookup)
            let name_var = Variable::new(var_counter);
            var_counter += 1;
            variables.insert(name.clone(), name_var);
            builder.declare_var(name_var, cranelift_ty);
            builder.def_var(name_var, block_param);
        }

        // Cache for function refs and global values
        let mut func_ref_cache: std::collections::HashMap<String, FuncRef> =
            std::collections::HashMap::new();
        let mut global_value_cache: std::collections::HashMap<usize, GlobalValue> =
            std::collections::HashMap::new();

        // Stack slots for alloca and stored values for load/store
        let mut stack_slots: std::collections::HashMap<String, cranelift::codegen::ir::StackSlot> =
            std::collections::HashMap::new();
        // Map from ptr name to (cranelift Value, Type)
        let mut stored_cranelift_values: std::collections::HashMap<String, (cranelift::prelude::Value, Type)> =
            std::collections::HashMap::new();

        // First pass: collect all labels and create blocks
        let mut label_to_block: std::collections::HashMap<String, cranelift::codegen::ir::Block> =
            std::collections::HashMap::new();
        for instr in &func.instructions {
            if let Instruction::Label(name) = instr {
                let block = builder.create_block();
                label_to_block.insert(name.clone(), block);
            }
        }

        // Pre-process: collect Phi node information
        // Map from label -> list of (result_var, incoming_value)
        let mut phi_info: std::collections::HashMap<String, Vec<(String, String)>> =
            std::collections::HashMap::new();
        for instr in &func.instructions {
            if let Instruction::Phi { result, incoming } = instr {
                for (val, label) in incoming {
                    phi_info.entry(label.clone())
                        .or_default()
                        .push((result.0.clone(), val.0.clone()));
                }
            }
        }

        // Store label_to_block and phi_info for use in compile_instruction
        let label_to_block_ref = &label_to_block;
        let phi_info_ref = &phi_info;

        // Track current label (for phi handling)
        let mut current_label: Option<String> = None;

        // Process instructions
        let func_name = &func.name;
        for instr in &func.instructions {
            // Track current label for phi handling
            if let Instruction::Label(name) = instr {
                current_label = Some(name.clone());
            }

            self.compile_instruction_with_control_flow(
                &mut builder,
                instr,
                func_name,
                &mut variables,
                &mut var_counter,
                &mut func_ref_cache,
                &mut global_value_cache,
                &mut stack_slots,
                &mut stored_cranelift_values,
                label_to_block_ref,
                phi_info_ref,
                &current_label,
            )?;
        }

        // Seal all blocks now that all predecessors are known
        for block in label_to_block.values() {
            builder.seal_block(*block);
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

    /// Compile a single instruction with control flow support
    fn compile_instruction_with_control_flow(
        &self,
        builder: &mut FunctionBuilder,
        instr: &Instruction,
        func_name: &str,
        variables: &mut std::collections::HashMap<String, Variable>,
        var_counter: &mut usize,
        func_ref_cache: &mut std::collections::HashMap<String, FuncRef>,
        global_value_cache: &mut std::collections::HashMap<usize, GlobalValue>,
        stack_slots: &mut std::collections::HashMap<String, cranelift::codegen::ir::StackSlot>,
        stored_cranelift_values: &mut std::collections::HashMap<String, (cranelift::prelude::Value, Type)>,
        label_to_block: &std::collections::HashMap<String, cranelift::codegen::ir::Block>,
        phi_info: &std::collections::HashMap<String, Vec<(String, String)>>,
        current_label: &Option<String>,
    ) -> Result<(), String> {
        match instr {
            Instruction::Jump(target_label) => {
                // Before jumping, define phi result variables for this predecessor
                if let Some(label) = current_label {
                    if let Some(phi_defs) = phi_info.get(label) {
                        for (result_var, incoming_val) in phi_defs {
                            // Load the incoming value
                            let val = self.load_variable_by_name(builder, incoming_val, variables)?;
                            // Get the type from the loaded value
                            let val_type = builder.func.dfg.value_type(val);

                            // Get or create the Variable for the result
                            let var = if let Some(&existing_var) = variables.get(result_var) {
                                existing_var
                            } else {
                                let new_var = Variable::new(*var_counter);
                                *var_counter += 1;
                                variables.insert(result_var.clone(), new_var);
                                builder.declare_var(new_var, val_type);
                                new_var
                            };

                            // Define the variable with this value
                            builder.def_var(var, val);
                        }
                    }
                }

                if let Some(&target_block) = label_to_block.get(target_label) {
                    builder.ins().jump(target_block, &[]);
                }
            }
            Instruction::Branch { cond, then_label, else_label } => {
                let cond_val = self.load_variable(builder, cond, variables)?;
                // Compare with zero (false)
                let zero = builder.ins().iconst(types::I64, 0);
                let cond_i8 = builder.ins().icmp(
                    cranelift::prelude::IntCC::NotEqual,
                    cond_val,
                    zero,
                );
                let then_block = *label_to_block.get(then_label)
                    .ok_or_else(|| format!("Label {} not found", then_label))?;
                let else_block = *label_to_block.get(else_label)
                    .ok_or_else(|| format!("Label {} not found", else_label))?;
                builder.ins().brif(cond_i8, then_block, &[], else_block, &[]);
            }
            Instruction::Label(name) => {
                if let Some(&block) = label_to_block.get(name) {
                    builder.switch_to_block(block);
                    // Don't seal immediately - we'll seal all blocks at the end
                    // This allows Cranelift to properly handle back-edges in loops
                }
            }
            _ => {
                // Handle other instructions using the original method
                self.compile_instruction(
                    builder,
                    instr,
                    func_name,
                    variables,
                    var_counter,
                    func_ref_cache,
                    global_value_cache,
                    stack_slots,
                    stored_cranelift_values,
                )?;
            }
        }
        Ok(())
    }

    /// Compile a single instruction
    fn compile_instruction(
        &self,
        builder: &mut FunctionBuilder,
        instr: &Instruction,
        func_name: &str,
        variables: &mut std::collections::HashMap<String, Variable>,
        var_counter: &mut usize,
        func_ref_cache: &mut std::collections::HashMap<String, FuncRef>,
        global_value_cache: &mut std::collections::HashMap<usize, GlobalValue>,
        stack_slots: &mut std::collections::HashMap<String, cranelift::codegen::ir::StackSlot>,
        stored_cranelift_values: &mut std::collections::HashMap<String, (cranelift::prelude::Value, Type)>,
    ) -> Result<(), String> {
        match instr {
            Instruction::Const { result, value, ty } => {
                let (val, cranelift_ty) = match ty {
                    IRType::I64 => {
                        let n: i64 = value.parse().unwrap_or(0);
                        (builder.ins().iconst(types::I64, n), types::I64)
                    }
                    IRType::F64 => {
                        let n: f64 = value.parse().unwrap_or(0.0);
                        (builder.ins().f64const(n), types::F64)
                    }
                    IRType::Bool => {
                        let b = value == "true";
                        // Store bool as I64 for consistency with comparison results
                        // and to avoid type mismatch with logical operators
                        (builder.ins().iconst(types::I64, i64::from(b)), types::I64)
                    }
                    _ => (builder.ins().iconst(types::I64, 0), types::I64),
                };
                self.store_variable(builder, result, val, variables, var_counter, cranelift_ty);
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

                // Load arguments and convert types if needed
                let sig = self.func_sigs.get(func_name)
                    .ok_or_else(|| format!("Signature not found for function {}", func_name))?;

                let arg_vals: Vec<Value> = args
                    .iter()
                    .enumerate()
                    .map(|(i, a)| -> Result<Value, String> {
                        let val = self.load_variable(builder, a, variables)?;
                        // Convert argument type to match function signature
                        let expected_type = sig.params.get(i)
                            .map(|p| p.value_type)
                            .unwrap_or(types::I64);
                        let val_type = builder.func.dfg.value_type(val);

                        if val_type != expected_type {
                            // Need type conversion
                            match (val_type, expected_type) {
                                (types::I8, types::I64) => {
                                    // Extend bool to i64
                                    Ok(builder.ins().uextend(types::I64, val))
                                }
                                (types::I64, types::F64) => {
                                    // Convert int to float
                                    Ok(builder.ins().fcvt_from_sint(types::F64, val))
                                }
                                (types::F64, types::I64) => {
                                    // Convert float to int
                                    Ok(builder.ins().fcvt_to_sint_sat(types::I64, val))
                                }
                                _ => Ok(val), // No conversion
                            }
                        } else {
                            Ok(val)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Make the call
                let call_val = builder.ins().call(func_ref, &arg_vals);

                // Store result if any
                if let Some(result) = result {
                    // Get the return value (first return value)
                    let results = builder.inst_results(call_val);
                    if !results.is_empty() {
                        let ret_val = results[0];

                        // Get return type from function signature
                        let sig = self.func_sigs.get(func_name)
                            .ok_or_else(|| format!("Signature not found for function {}", func_name))?;
                        let ret_type = sig.returns.first()
                            .map(|p| p.value_type)
                            .unwrap_or(types::I64);

                        self.store_variable(builder, result, ret_val, variables, var_counter, ret_type);
                    }
                }
            }
            // Jump, Branch, and Label are handled in compile_instruction_with_control_flow
            // but we need to handle them here for completeness
            Instruction::Jump(_) | Instruction::Branch { .. } | Instruction::Label(_) => {
                // Already handled in compile_instruction_with_control_flow
            }
            Instruction::Alloca { result, ty } => {
                // Create a stack slot for the variable
                let size = match ty {
                    IRType::I8 | IRType::U8 | IRType::Bool | IRType::Char => 1,
                    IRType::I16 | IRType::U16 | IRType::F8 | IRType::F16 => 2,
                    IRType::I32 | IRType::U32 | IRType::F32 => 4,
                    IRType::I64 | IRType::U64 | IRType::F64 => 8,
                    IRType::I128 | IRType::U128 | IRType::F128 => 16,
                    IRType::String | IRType::Ptr(_) | IRType::Object => 8, // pointer size
                    IRType::Void => 0,
                };
                let slot = builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData::new(
                    cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                    size,
                    0,
                ));
                stack_slots.insert(result.0.clone(), slot);
            }
            Instruction::Store { value, ptr } => {
                // Get the value to store
                let val = self.load_variable(builder, value, variables)?;
                let val_type = builder.func.dfg.value_type(val);

                // Store in our tracking map for later loads
                stored_cranelift_values.insert(ptr.0.clone(), (val, val_type));

                // Also store to stack slot if we have one
                if let Some(&slot) = stack_slots.get(&ptr.0) {
                    let addr = builder.ins().stack_addr(self.pointer_type, slot, 0);
                    builder.ins().store(cranelift::codegen::ir::MemFlags::trusted(), val, addr, 0);
                }

                // For loop variables: define a Cranelift variable for the pointer
                // This allows use_var to get the correct value with phi nodes in loops
                let ptr_name = &ptr.0;
                if let Some(&existing_var) = variables.get(ptr_name) {
                    // Reuse existing variable - Cranelift will handle phi nodes
                    builder.def_var(existing_var, val);
                } else {
                    // Create new variable for this storage location
                    let var = Variable::new(*var_counter);
                    *var_counter += 1;
                    variables.insert(ptr_name.clone(), var);
                    builder.declare_var(var, val_type);
                    builder.def_var(var, val);
                }
            }
            Instruction::Load { result, ptr } => {
                let ptr_name = &ptr.0;

                // First, try to use Cranelift variable (handles loops with phi nodes)
                if let Some(&var) = variables.get(ptr_name) {
                    let val = builder.use_var(var);
                    let val_type = builder.func.dfg.value_type(val);
                    self.store_variable(builder, result, val, variables, var_counter, val_type);
                } else if let Some((val, val_type)) = stored_cranelift_values.get(ptr_name) {
                    // Fallback to stored values
                    self.store_variable(builder, result, *val, variables, var_counter, *val_type);
                } else if let Some(&slot) = stack_slots.get(ptr_name) {
                    // Load from stack slot
                    let addr = builder.ins().stack_addr(self.pointer_type, slot, 0);
                    let val = builder.ins().load(
                        self.pointer_type,
                        cranelift::codegen::ir::MemFlags::trusted(),
                        addr,
                        0,
                    );
                    self.store_variable(builder, result, val, variables, var_counter, self.pointer_type);
                } else {
                    // Default: return 0
                    let val = builder.ins().iconst(types::I64, 0);
                    self.store_variable(builder, result, val, variables, var_counter, types::I64);
                }
            }
            Instruction::Phi { result, incoming: _ } => {
                // Phi node: we need to declare the result variable if not already declared.
                // The actual values are set in each predecessor block's Jump instruction.
                // Check if the variable already exists (it might have been defined in predecessor jumps)
                if !variables.contains_key(&result.0) {
                    // Declare with a placeholder type - will be re-typed when first defined
                    let var = Variable::new(*var_counter);
                    *var_counter += 1;
                    variables.insert(result.0.clone(), var);
                    // Don't call declare_var here as we don't know the type yet
                    // The type will be set when def_var is called in the predecessor blocks
                }
            }
            Instruction::StringConcat { result, left, left_type, right, right_type } => {
                // Determine which runtime function to call
                let func_name = match (left_type, right_type) {
                    (ConcatType::String, ConcatType::String) => "xin_str_concat_ss",
                    (ConcatType::String, ConcatType::Int) => "xin_str_concat_si",
                    (ConcatType::Int, ConcatType::String) => "xin_str_concat_is",
                    (ConcatType::String, ConcatType::Float) => "xin_str_concat_sf",
                    (ConcatType::Float, ConcatType::String) => "xin_str_concat_fs",
                    (ConcatType::String, ConcatType::Bool) => "xin_str_concat_sb",
                    (ConcatType::Bool, ConcatType::String) => "xin_str_concat_bs",
                    _ => "xin_str_concat_ss",
                };

                let left_val = self.load_variable(builder, left, variables)?;
                let right_val = self.load_variable(builder, right, variables)?;

                // Get or create the function reference
                let func_ref = if let Some(fr) = func_ref_cache.get(func_name) {
                    *fr
                } else {
                    let func_id = *self.extern_func_ids.get(func_name)
                        .ok_or_else(|| format!("Function '{}' not declared", func_name))?;
                    let sig = self.func_sigs.get(func_name)
                        .ok_or_else(|| format!("Signature not found for function '{}'", func_name))?
                        .clone();
                    let sig_ref = builder.func.import_signature(sig);
                    let user_func_name = builder.func.declare_imported_user_function(UserExternalName {
                        namespace: 0,
                        index: func_id.as_u32(),
                    });
                    let fr = builder.import_function(ExtFuncData {
                        name: ExternalName::user(user_func_name),
                        signature: sig_ref,
                        colocated: true,
                    });
                    func_ref_cache.insert(func_name.to_string(), fr);
                    fr
                };

                let call_val = builder.ins().call(func_ref, &[left_val, right_val]);
                let ret_val = builder.inst_results(call_val)[0];
                self.store_variable(builder, result, ret_val, variables, var_counter, self.pointer_type);
            }
            Instruction::StringFree { value } => {
                let val = self.load_variable(builder, value, variables)?;

                let func_ref = if let Some(fr) = func_ref_cache.get("xin_str_free") {
                    *fr
                } else {
                    let func_id = *self.extern_func_ids.get("xin_str_free")
                        .ok_or_else(|| "Function 'xin_str_free' not declared".to_string())?;
                    let sig = self.func_sigs.get("xin_str_free")
                        .ok_or_else(|| "Signature not found for function 'xin_str_free'".to_string())?
                        .clone();
                    let sig_ref = builder.func.import_signature(sig);
                    let user_func_name = builder.func.declare_imported_user_function(UserExternalName {
                        namespace: 0,
                        index: func_id.as_u32(),
                    });
                    let fr = builder.import_function(ExtFuncData {
                        name: ExternalName::user(user_func_name),
                        signature: sig_ref,
                        colocated: true,
                    });
                    func_ref_cache.insert("xin_str_free".to_string(), fr);
                    fr
                };

                builder.ins().call(func_ref, &[val]);
            }
            Instruction::ToString { result, value, from_type } => {
                let val = self.load_variable(builder, value, variables)?;

                let func_name = match from_type {
                    IRType::I64 => "xin_int_to_str",
                    IRType::F64 => "xin_float_to_str",
                    IRType::Bool => "xin_bool_to_str",
                    _ => "xin_int_to_str",
                };

                let func_ref = if let Some(fr) = func_ref_cache.get(func_name) {
                    *fr
                } else {
                    let func_id = *self.extern_func_ids.get(func_name)
                        .ok_or_else(|| format!("Function '{}' not declared", func_name))?;
                    let sig = self.func_sigs.get(func_name)
                        .ok_or_else(|| format!("Signature not found for function '{}'", func_name))?
                        .clone();
                    let sig_ref = builder.func.import_signature(sig);
                    let user_func_name = builder.func.declare_imported_user_function(UserExternalName {
                        namespace: 0,
                        index: func_id.as_u32(),
                    });
                    let fr = builder.import_function(ExtFuncData {
                        name: ExternalName::user(user_func_name),
                        signature: sig_ref,
                        colocated: true,
                    });
                    func_ref_cache.insert(func_name.to_string(), fr);
                    fr
                };

                let call_val = builder.ins().call(func_ref, &[val]);
                let ret_val = builder.inst_results(call_val)[0];
                self.store_variable(builder, result, ret_val, variables, var_counter, self.pointer_type);
            }
            // Array instructions
            Instruction::ArrayNew { result, capacity } => {
                // Call xin_array_new(capacity) -> xin_array*
                let capacity_val = builder.ins().iconst(types::I64, *capacity as i64);

                let func_ref = self.get_or_create_func_ref(
                    builder,
                    "xin_array_new",
                    func_ref_cache,
                )?;

                let call_val = builder.ins().call(func_ref, &[capacity_val]);
                let arr_ptr = builder.inst_results(call_val)[0];
                self.store_variable(builder, result, arr_ptr, variables, var_counter, self.pointer_type);
            }
            Instruction::ArrayGet { result, array, index } => {
                // Call xin_array_get(array, index) -> void*
                let arr_val = self.load_variable(builder, array, variables)?;
                let idx_val = self.load_variable(builder, index, variables)?;

                let func_ref = self.get_or_create_func_ref(
                    builder,
                    "xin_array_get",
                    func_ref_cache,
                )?;

                let call_val = builder.ins().call(func_ref, &[arr_val, idx_val]);
                let val = builder.inst_results(call_val)[0];
                self.store_variable(builder, result, val, variables, var_counter, self.pointer_type);
            }
            Instruction::ArraySet { array, index, value } => {
                // Call xin_array_set(array, index, value) -> void
                let arr_val = self.load_variable(builder, array, variables)?;
                let idx_val = self.load_variable(builder, index, variables)?;
                let val = self.load_variable(builder, value, variables)?;

                let func_ref = self.get_or_create_func_ref(
                    builder,
                    "xin_array_set",
                    func_ref_cache,
                )?;

                builder.ins().call(func_ref, &[arr_val, idx_val, val]);
            }
            Instruction::ArrayPush { array, value } => {
                // Call xin_array_push(array, value) -> void
                let arr_val = self.load_variable(builder, array, variables)?;
                let val = self.load_variable(builder, value, variables)?;

                let func_ref = self.get_or_create_func_ref(
                    builder,
                    "xin_array_push",
                    func_ref_cache,
                )?;

                builder.ins().call(func_ref, &[arr_val, val]);
            }
            Instruction::ArrayPop { result, array } => {
                // Call xin_array_pop(array) -> void*
                let arr_val = self.load_variable(builder, array, variables)?;

                let func_ref = self.get_or_create_func_ref(
                    builder,
                    "xin_array_pop",
                    func_ref_cache,
                )?;

                let call_val = builder.ins().call(func_ref, &[arr_val]);
                let val = builder.inst_results(call_val)[0];
                self.store_variable(builder, result, val, variables, var_counter, self.pointer_type);
            }
            Instruction::ArrayLen { result, array } => {
                // Call xin_array_len(array) -> int64_t
                let arr_val = self.load_variable(builder, array, variables)?;

                let func_ref = self.get_or_create_func_ref(
                    builder,
                    "xin_array_len",
                    func_ref_cache,
                )?;

                let call_val = builder.ins().call(func_ref, &[arr_val]);
                let len = builder.inst_results(call_val)[0];
                self.store_variable(builder, result, len, variables, var_counter, types::I64);
            }
            Instruction::Break | Instruction::Continue => {
                // TODO: Implement break/continue codegen (Task 3.3)
                // These require loop context tracking to jump to the appropriate label
            }
            Instruction::TypeCast { result, value, from_type, to_type } => {
                let val = self.load_variable(builder, value, variables)?;
                let cast_val = self.emit_type_cast(builder, val, from_type, to_type);
                let cranelift_ty = self.convert_ir_type(to_type);
                self.store_variable(builder, result, cast_val, variables, var_counter, cranelift_ty);
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

    /// Get or create a function reference for external function calls
    fn get_or_create_func_ref(
        &self,
        builder: &mut FunctionBuilder,
        func_name: &str,
        func_ref_cache: &mut std::collections::HashMap<String, FuncRef>,
    ) -> Result<FuncRef, String> {
        if let Some(&fr) = func_ref_cache.get(func_name) {
            return Ok(fr);
        }

        let func_id = *self.extern_func_ids.get(func_name)
            .ok_or_else(|| format!("Function '{}' not declared", func_name))?;
        let sig = self.func_sigs.get(func_name)
            .ok_or_else(|| format!("Signature not found for function '{}'", func_name))?
            .clone();
        let sig_ref = builder.func.import_signature(sig);
        let user_func_name = builder.func.declare_imported_user_function(UserExternalName {
            namespace: 0,
            index: func_id.as_u32(),
        });
        let fr = builder.import_function(ExtFuncData {
            name: ExternalName::user(user_func_name),
            signature: sig_ref,
            colocated: true,
        });
        func_ref_cache.insert(func_name.to_string(), fr);
        Ok(fr)
    }

    /// Load a variable by name (string)
    fn load_variable_by_name(
        &self,
        builder: &mut FunctionBuilder,
        name: &str,
        variables: &std::collections::HashMap<String, Variable>,
    ) -> Result<Value, String> {
        if let Some(var) = variables.get(name) {
            Ok(builder.use_var(*var))
        } else {
            // Return a default value for undefined variables
            Ok(builder.ins().iconst(types::I64, 0))
        }
    }

    /// Store a variable by name (string)
    fn store_variable_by_name(
        &self,
        builder: &mut FunctionBuilder,
        name: &str,
        value: Value,
        variables: &mut std::collections::HashMap<String, Variable>,
        var_counter: &mut usize,
        ty: Type,
    ) {
        let var = Variable::new(*var_counter);
        *var_counter += 1;
        variables.insert(name.to_string(), var);
        builder.declare_var(var, ty);
        builder.def_var(var, value);
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
            IRType::I8 => types::I8,
            IRType::I16 => types::I16,
            IRType::I32 => types::I32,
            IRType::I64 => types::I64,
            IRType::I128 => types::I128,
            IRType::U8 => types::I8,
            IRType::U16 => types::I16,
            IRType::U32 => types::I32,
            IRType::U64 => types::I64,
            IRType::U128 => types::I128,
            IRType::F8 => types::F32,  // No F8 in cranelift, use F32
            IRType::F16 => types::F32, // No F16 in cranelift, use F32
            IRType::F32 => types::F32,
            IRType::F64 => types::F64,
            IRType::F128 => types::F64, // No F128 in cranelift, use F64
            IRType::Char => types::I32, // Char is represented as I32 (Unicode code point)
            IRType::Bool => types::I64, // Use I64 for bool to match logical operators
            IRType::String => self.pointer_type,
            IRType::Void => panic!("Void type should not be converted to Cranelift type"),
            IRType::Ptr(_) => self.pointer_type,
            IRType::Object => self.pointer_type, // Object types are pointers
        }
    }

    /// Alias for convert_type for clarity
    fn convert_ir_type(&self, ty: &IRType) -> Type {
        self.convert_type(ty)
    }

    /// Emit type cast instruction
    fn emit_type_cast(
        &self,
        builder: &mut FunctionBuilder,
        value: Value,
        from_type: &IRType,
        to_type: &IRType,
    ) -> Value {
        let from = self.convert_type(from_type);
        let to = self.convert_type(to_type);

        // Handle integer to integer conversions
        if from.is_int() && to.is_int() {
            let from_bits = from.bits();
            let to_bits = to.bits();

            if to_bits > from_bits {
                // Extension: sign-extend for signed, zero-extend for unsigned
                if Self::is_signed_int(from_type) {
                    builder.ins().sextend(to, value)
                } else {
                    builder.ins().uextend(to, value)
                }
            } else if to_bits < from_bits {
                // Truncation
                builder.ins().ireduce(to, value)
            } else {
                // Same size, no conversion needed
                value
            }
        }
        // Handle float to float conversions
        else if from.is_float() && to.is_float() {
            if to.bits() > from.bits() {
                builder.ins().fpromote(to, value)
            } else if to.bits() < from.bits() {
                builder.ins().fdemote(to, value)
            } else {
                value
            }
        }
        // Handle integer to float conversions
        else if from.is_int() && to.is_float() {
            if Self::is_signed_int(from_type) {
                builder.ins().fcvt_from_sint(to, value)
            } else {
                builder.ins().fcvt_from_uint(to, value)
            }
        }
        // Handle float to integer conversions
        else if from.is_float() && to.is_int() {
            if Self::is_signed_int(to_type) {
                builder.ins().fcvt_to_sint_sat(to, value)
            } else {
                builder.ins().fcvt_to_uint_sat(to, value)
            }
        }
        // Default: just return the value
        else {
            value
        }
    }

    /// Check if an IR type is a signed integer
    fn is_signed_int(ty: &IRType) -> bool {
        matches!(ty, IRType::I8 | IRType::I16 | IRType::I32 | IRType::I64 | IRType::I128)
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