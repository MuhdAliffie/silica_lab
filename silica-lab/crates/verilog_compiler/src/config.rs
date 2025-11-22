use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use which::which;

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub iverilog_path: PathBuf,
    pub verilator_path: Option<PathBuf>,
    pub vvp_path: Option<PathBuf>,
    pub gtkwave_path: Option<PathBuf>,
    pub output_dir: PathBuf,
    pub debug: bool,
}

impl CompilerConfig {
    /// Attempt to discover common Verilog tool executables on the host system.
    ///
    /// - Searches the user's PATH for `iverilog`, `verilator`, `vvp`, and `gtkwave`.
    /// - On Windows, the `which` crate handles `.exe` suffixes automatically.
    /// - If `iverilog` is not found, the returned `iverilog_path` will be
    ///   the bare string `iverilog` (so callers can still attempt to spawn it
    ///   and receive the runtime error). Optional tools are returned as `None`
    ///   when not discovered.
    pub fn discover() -> Self {
        let iverilog_path = which("iverilog").unwrap_or_else(|_| PathBuf::from("iverilog"));
        let verilator_path = which("verilator").ok();
        let vvp_path = which("vvp").ok();
        let gtkwave_path = which("gtkwave").ok();
        let output_dir = PathBuf::from("target/verilog_out");
        let debug = false;

        CompilerConfig {
            iverilog_path,
            verilator_path,
            vvp_path,
            gtkwave_path,
            output_dir,
            debug,
        }
    }

    /// Ensure required tools and directories are available for running Icarus Verilog.
    pub fn validate_for_iverilog(&self) -> Result<()> {
        // iverilog is required
        let iverilog = &self.iverilog_path;
        if !iverilog.as_path().exists() {
            bail!(
                "iverilog not found at {:?}. Install Icarus Verilog and/or set CompilerConfig.iverilog_path",
                iverilog
            );
        }
        if !iverilog.as_path().is_file() {
            bail!("iverilog path is not a file: {:?}", iverilog);
        }

        // vvp runtime must also be present to execute the simulation
        let vvp = match &self.vvp_path {
            Some(p) => p.clone(),
            None => which("vvp").context("vvp runtime not found on PATH; install Icarus Verilog or set CompilerConfig.vvp_path")?,
        };
        if !vvp.is_file() {
            bail!("vvp path is not a file: {:?}", vvp);
        }

        // Ensure configured output_dir exists (independent from specific command outputs)
        std::fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("Failed to create output directory {:?}", self.output_dir))?;
        Ok(())
    }

    /// Ensure Verilator is available when running Verilator-based tasks.
    pub fn validate_for_verilator(&self) -> Result<()> {
        let verilator = self.verilator_path.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "Verilator is not configured. Install it or set CompilerConfig.verilator_path"
            )
        })?;

        if !verilator.as_path().exists() {
            bail!("verilator not found at {:?}", verilator);
        }
        if !verilator.as_path().is_file() {
            bail!("verilator path is not a file: {:?}", verilator);
        }

        std::fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("Failed to create output directory {:?}", self.output_dir))?;
        Ok(())
    }

    /// Ensure GTKWave is available before attempting to launch it.
    pub fn validate_for_gtkwave(&self) -> Result<()> {
        let gtkwave = match &self.gtkwave_path {
            Some(p) => p.clone(),
            None => which("gtkwave").context(
                "gtkwave not found on PATH; install GTKWave or set CompilerConfig.gtkwave_path",
            )?,
        };

        if !gtkwave.is_file() {
            bail!("gtkwave path is not a file: {:?}", gtkwave);
        }
        Ok(())
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self::discover()
    }
}
