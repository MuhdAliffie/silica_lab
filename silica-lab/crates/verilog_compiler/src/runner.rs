use crate::config::CompilerConfig;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn run_vvp(sim_executable: &Path, vcd_path: &Path, config: &CompilerConfig) -> Result<()> {
    // Validate toolchain components needed by vvp (through iverilog validation)
    config
        .validate_for_iverilog()
        .context("Invalid compiler configuration for Icarus Verilog runtime (vvp)")?;
    // Ensure the VCD parent directory exists if a path with parent was provided
    if let Some(parent) = vcd_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create VCD parent directory {:?}", parent))?;
        }
    }

    let mut cmd = if let Some(path) = &config.vvp_path {
        Command::new(path)
    } else {
        Command::new("vvp")
    };
    cmd.arg(sim_executable);

    if config.debug {
        println!("Running vvp command: {:?}", cmd);
    }

    let output_status = cmd.output().context("Failed to execute vvp")?;

    if !output_status.status.success() {
        let stdout = String::from_utf8_lossy(&output_status.stdout);
        let stderr = String::from_utf8_lossy(&output_status.stderr);
        let code = output_status
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        anyhow::bail!(
            "Simulation failed (exit code {}):\nstdout:\n{}\nstderr:\n{}",
            code,
            stdout,
            stderr
        );
    }

    // Optionally move or parse vcd
    if vcd_path.exists() {
        println!("VCD file generated at {:?}", vcd_path);
    } else {
        eprintln!(
            "Warning: VCD file not found at {:?}. Ensure your testbench calls $dumpfile/$dumpvars.",
            vcd_path
        );
    }

    Ok(())
}

/// Launch GTKWave to view a VCD file.
///
/// If `background` is true, this will spawn the GTKWave process and return immediately.
/// Otherwise, it will block until GTKWave exits.
pub fn run_gtkwave(vcd_path: &Path, config: &CompilerConfig, background: bool) -> Result<()> {
    // Validate gtkwave availability
    config
        .validate_for_gtkwave()
        .context("Invalid configuration for GTKWave")?;

    // Ensure VCD exists before launching
    if !vcd_path.exists() {
        anyhow::bail!(
            "VCD file not found at {:?}. Run the simulation or check the dumpfile path.",
            vcd_path
        );
    }

    let mut cmd = if let Some(path) = &config.gtkwave_path {
        Command::new(path)
    } else {
        Command::new("gtkwave")
    };
    cmd.arg(vcd_path);

    if config.debug {
        println!("Launching GTKWave: {:?}", cmd);
    }

    if background {
        let _child = cmd.spawn().context("Failed to launch gtkwave")?;
        Ok(())
    } else {
        let status = cmd.status().context("Failed to run gtkwave")?;
        if !status.success() {
            anyhow::bail!("gtkwave exited with status: {:?}", status.code());
        }
        Ok(())
    }
}
