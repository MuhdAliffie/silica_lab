use crate::compiler::{icarus, verilator};
use crate::{config::CompilerConfig, parser::detect_language};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn compile(files: Vec<PathBuf>, output: &Path, config: &CompilerConfig) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!("No files provided for compilation");
    }

    let lang = detect_language(&files[0]);

    match lang {
        crate::parser::VerilogLang::Verilog => icarus::compile_iverilog(&files, output, config)?,
        crate::parser::VerilogLang::SystemVerilog => verilator::compile_verilator(&files, config)?,
    }

    Ok(())
}
