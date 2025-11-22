use crate::config::CompilerConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn compile_iverilog(files: &[PathBuf], output: &Path, config: &CompilerConfig) -> Result<()> {
    // Validate toolchain availability and configured output directory
    config
        .validate_for_iverilog()
        .context("Invalid compiler configuration for Icarus Verilog")?;
    let out_parent = output.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(out_parent)
        .with_context(|| format!("Failed to create output directory {:?}", out_parent))?;

    let mut cmd = Command::new(&config.iverilog_path);
    cmd.arg("-o").arg(output);
    for file in files {
        cmd.arg(file);
    }

    if config.debug {
        println!("Running iverilog command: {:?}", cmd);
    }

    let output_status = cmd.output().context("Failed to execute iverilog")?;

    if !output_status.status.success() {
        let stdout = String::from_utf8_lossy(&output_status.stdout);
        let stderr = String::from_utf8_lossy(&output_status.stderr);
        let code = output_status
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        anyhow::bail!(
            "Iverilog compilation failed (exit code {}):\nstdout:\n{}\nstderr:\n{}",
            code,
            stdout,
            stderr
        );
    }

    Ok(())
}
