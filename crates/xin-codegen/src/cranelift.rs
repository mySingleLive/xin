//! Cranelift code generator

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use xin_ir::{BinOp, Instruction, IRFunction, IRModule, IRType};

/// Code generator using Cranelift
pub struct CodeGenerator {
    module: JITModule,
}

impl CodeGenerator {
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();

        let isa_builder = cranelift_native::builder()
            .map_err(|e| format!("Failed to create ISA builder: {}", e))?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| format!("Failed to create ISA: {}", e))?;

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Ok(Self { module })
    }

    pub fn compile(&mut self, module: &IRModule) -> Result<(), String> {
        for func in &module.functions {
            self.compile_function(func)?;
        }
        Ok(())
    }

    fn compile_function(&mut self, func: &IRFunction) -> Result<(), String> {
        let pointer_type = self.module.target_config().pointer_type();

        // Create function signature
        let mut sig = self.module.make_signature();
        for (_, ty) in &func.params {
            sig.params.push(AbiParam::new(self.convert_type(ty)));
        }
        // Only add return type if not void
        if func.return_type != IRType::Void {
            sig.returns.push(AbiParam::new(self.convert_type(&func.return_type)));
        }

        // Declare function
        let func_id = self
            .module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Failed to declare function: {}", e))?;

        // Create function context
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;

        // Create function builder
        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Variables map
        let mut variables: std::collections::HashMap<String, Variable> = std::collections::HashMap::new();
        let mut var_counter = 0;

        // Process parameters
        for (name, _) in &func.params {
            let var = Variable::new(var_counter);
            var_counter += 1;
            variables.insert(name.clone(), var);
        }

        // First pass: collect all labels and create blocks
        let mut label_to_block: std::collections::HashMap<String, Block> = std::collections::HashMap::new();
        for instr in &func.instructions {
            if let Instruction::Label(name) = instr {
                let block = builder.create_block();
                label_to_block.insert(name.clone(), block);
            }
        }

        // Process instructions
        for instr in &func.instructions {
            self.compile_instruction(&mut builder, instr, &mut variables, &mut var_counter, pointer_type, &label_to_block)?;
        }

        // Seal all blocks that were created for labels
        for block in label_to_block.values() {
            builder.seal_block(*block);
        }

        builder.finalize();

        // Define function
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Failed to define function: {}", e))?;

        self.module.clear_context(&mut ctx);

        Ok(())
    }

    fn compile_instruction(
        &self,
        builder: &mut FunctionBuilder,
        instr: &Instruction,
        variables: &mut std::collections::HashMap<String, Variable>,
        var_counter: &mut usize,
        pointer_type: Type,
        label_to_block: &std::collections::HashMap<String, Block>,
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
                self.store_variable(builder, result, val, variables, var_counter);
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
                        let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::Le => {
                        let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::Ge => {
                        let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left_val, right_val);
                        builder.ins().uextend(types::I64, cmp)
                    }
                    BinOp::And => builder.ins().band(left_val, right_val),
                    BinOp::Or => builder.ins().bor(left_val, right_val),
                };
                self.store_variable(builder, result, val, variables, var_counter);
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
                // For now, just emit a placeholder
                // TODO: Implement proper function calls
                if let Some(result) = result {
                    let val = builder.ins().iconst(types::I64, 0);
                    self.store_variable(builder, result, val, variables, var_counter);
                }
            }
            Instruction::Jump(target_label) => {
                if let Some(&target_block) = label_to_block.get(target_label) {
                    builder.ins().jump(target_block, &[]);
                } else {
                    return Err(format!("Label {} not found", target_label));
                }
            }
            Instruction::Branch { cond, then_label, else_label } => {
                let cond_val = self.load_variable(builder, cond, variables)?;
                // Compare with zero (false)
                let zero = builder.ins().iconst(types::I64, 0);
                let cond_i8 = builder.ins().icmp(IntCC::NotEqual, cond_val, zero);

                let then_block = *label_to_block.get(then_label)
                    .ok_or_else(|| format!("Label {} not found", then_label))?;
                let else_block = *label_to_block.get(else_label)
                    .ok_or_else(|| format!("Label {} not found", else_label))?;

                builder.ins().brif(cond_i8, then_block, &[], else_block, &[]);
            }
            Instruction::Label(name) => {
                if let Some(&block) = label_to_block.get(name) {
                    builder.switch_to_block(block);
                }
            }
            _ => {}
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
            // Create a new variable for constants
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
    ) {
        let var = Variable::new(*var_counter);
        *var_counter += 1;
        variables.insert(result.0.clone(), var);
        builder.declare_var(var, types::I64);
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
            IRType::F8 => types::F32,
            IRType::F16 => types::F32,
            IRType::F32 => types::F32,
            IRType::F64 => types::F64,
            IRType::F128 => types::F64,
            IRType::Char => types::I32,
            IRType::Bool => types::I8,
            IRType::String => types::I64, // String as pointer
            IRType::Void => panic!("Void type should not be converted to Cranelift type"),
            IRType::Ptr(_) => types::I64,
            IRType::Object => types::I64, // Object types are pointers
        }
    }

    pub fn finalize(&mut self) -> Result<(), String> {
        self.module
            .finalize_definitions()
            .map_err(|e| format!("Failed to finalize: {}", e))
    }

    pub fn get_function_address(&self, name: &str) -> Result<*const u8, String> {
        let func_or_data_id = self
            .module
            .get_name(name)
            .ok_or_else(|| format!("Function {} not found", name))?;

        let func_id = match func_or_data_id {
            cranelift_module::FuncOrDataId::Func(id) => id,
            cranelift_module::FuncOrDataId::Data(_) => {
                return Err(format!("{} is not a function", name));
            }
        };

        Ok(self.module.get_finalized_function(func_id))
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create code generator")
    }
}