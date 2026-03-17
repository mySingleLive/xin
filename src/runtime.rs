//! Runtime embedding module
//!
//! This module embeds the C runtime source code into the compiler binary
//! and provides functions to write it to a temporary file for linking.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the embedded runtime source code
pub fn get_runtime_source() -> &'static str {
    include_str!("../runtime/runtime.c")
}

/// Write the runtime source to a temporary file
///
/// Uses process ID and timestamp to avoid conflicts when multiple
/// xin compile processes run concurrently.
pub fn write_runtime_to_temp() -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir();
    let pid = std::process::id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_millis();
    let runtime_path = temp_dir.join(format!("xin_runtime_{}_{}.c", pid, timestamp));
    std::fs::write(&runtime_path, get_runtime_source())
        .map_err(|e| format!("Failed to write runtime: {}", e))?;
    Ok(runtime_path)
}