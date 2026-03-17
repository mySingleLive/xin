//! Linker wrapper for linking object files into executables

use std::path::Path;
use std::process::Command;

/// Linker wrapper that uses the system C compiler
pub struct Linker {
    c_compiler: String,
}

impl Linker {
    /// Create a new linker instance, detecting the available C compiler
    pub fn new() -> Result<Self, String> {
        let compilers = ["cc", "gcc", "clang"];
        for compiler in &compilers {
            if which::which(compiler).is_ok() {
                return Ok(Self {
                    c_compiler: compiler.to_string(),
                });
            }
        }
        Err("No C compiler found. Please install cc, gcc, or clang.".to_string())
    }

    /// Link an object file with the runtime into an executable
    pub fn link(
        &self,
        obj_path: &Path,
        runtime_path: &Path,
        output: &Path,
    ) -> Result<(), String> {
        let status = Command::new(&self.c_compiler)
            .arg(obj_path)
            .arg(runtime_path)
            .arg("-o")
            .arg(output)
            .status()
            .map_err(|e| format!("Failed to run linker: {}", e))?;

        if !status.success() {
            return Err(format!(
                "Linker failed with exit code: {:?}",
                status.code()
            ));
        }
        Ok(())
    }
}

impl Default for Linker {
    fn default() -> Self {
        Self::new().expect("Failed to create linker")
    }
}