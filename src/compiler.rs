//! Main compiler orchestration

use std::path::Path;

pub struct Compiler {
    // TODO: 添加编译器状态
}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, input: &Path) -> anyhow::Result<()> {
        // TODO: 实现编译流程
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}