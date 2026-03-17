//! Main compiler orchestration

use std::path::{Path, PathBuf};

use xin_codegen::AOTCodeGenerator;
use xin_ir::IRBuilder;
use xin_lexer::Lexer;
use xin_parser::Parser;
use xin_semantic::TypeChecker;

use crate::linker::Linker;
use crate::runtime;

pub struct Compiler {
    emit_ir: bool,
    output: Option<PathBuf>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            emit_ir: false,
            output: None,
        }
    }

    pub fn with_emit_ir(mut self, emit: bool) -> Self {
        self.emit_ir = emit;
        self
    }

    pub fn with_output(mut self, output: PathBuf) -> Self {
        self.output = Some(output);
        self
    }

    pub fn compile(&self, input: &Path) -> anyhow::Result<()> {
        // Read source file
        let source = std::fs::read_to_string(input)?;

        // Lexing
        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer)?;

        // Parsing
        let ast = parser.parse()?;

        // Type checking
        let mut type_checker = TypeChecker::new();
        if let Err(errors) = type_checker.check(&ast) {
            for error in errors {
                eprintln!("Error: {}", error.message);
            }
            anyhow::bail!("Type checking failed");
        }

        // IR generation
        let mut ir_builder = IRBuilder::new();
        let ir_module = ir_builder.build(&ast);

        if self.emit_ir {
            println!("IR Module:");
            for func in &ir_module.functions {
                println!("  fn {}:", func.name);
                for instr in &func.instructions {
                    println!("    {:?}", instr);
                }
            }
            println!("  extern functions: {:?}", ir_module.extern_functions);
            println!("  strings: {:?}", ir_module.strings);
        }

        // AOT Code generation
        let mut codegen = AOTCodeGenerator::new()
            .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;
        codegen
            .compile(&ir_module)
            .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;

        // Emit object file
        let obj_bytes = codegen
            .emit_object()
            .map_err(|e| anyhow::anyhow!("Object emission error: {}", e))?;

        // Determine output path
        let output = match &self.output {
            Some(path) => path.clone(),
            None => {
                let input_name = input.file_stem().unwrap_or(std::ffi::OsStr::new("a.out"));
                PathBuf::from(input_name)
            }
        };

        // Write object file to temp
        let temp_dir = std::env::temp_dir();
        let pid = std::process::id();
        let obj_path = temp_dir.join(format!("xin_output_{}.o", pid));
        std::fs::write(&obj_path, &obj_bytes)?;

        // Write runtime to temp
        let runtime_path = runtime::write_runtime_to_temp()
            .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;

        // Link
        let linker = Linker::new()
            .map_err(|e| anyhow::anyhow!("Linker error: {}", e))?;
        linker
            .link(&obj_path, &runtime_path, &output)
            .map_err(|e| anyhow::anyhow!("Link error: {}", e))?;

        // Cleanup temp files
        let _ = std::fs::remove_file(&obj_path);
        let _ = std::fs::remove_file(&runtime_path);

        println!("Compiled successfully to: {:?}", output);
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}