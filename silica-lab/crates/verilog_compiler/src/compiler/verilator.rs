use crate::config::CompilerConfig;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Run Verilator in lint mode over the provided files.
///
/// This does not generate a simulation; it simply lints the sources using
/// `verilator --lint-only -Wall ...`. If Verilator isn't configured, this
/// returns an actionable error message.
pub fn compile_verilator(files: &[PathBuf], config: &CompilerConfig) -> Result<()> {
    // Validate Verilator availability and configured directories
    config
        .validate_for_verilator()
        .context("Invalid compiler configuration for Verilator")?;

    let verilator_path = config.verilator_path.as_ref().expect("validated above");

    let mut cmd = Command::new(verilator_path);
    // Lint-only is cross-platform and fast; -Wall for useful diagnostics.
    // Verilator requires an explicit timing choice when delays are present; default to --no-timing for lint.
    // Suppress a few common TB warnings to avoid non-zero exit purely due to warnings.
    cmd.arg("--lint-only")
        .arg("-Wall")
        .arg("--no-timing")
        .arg("-Wno-STMTDLY")
        .arg("-Wno-UNUSEDSIGNAL")
        .arg("-Wno-UNOPTFLAT");
    for f in files {
        cmd.arg(f);
    }

    if config.debug {
        println!("Running verilator command: {:?}", cmd);
    }

    let output = cmd.output().context("Failed to execute verilator")?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If Verilator exits non-zero due only to warnings (common in lint-only flows),
        // tolerate it as success to keep CI/dev flows smooth.
        let has_fatal_errors = stderr.contains("%Error-");
        let exiting_due_to_warnings =
            stderr.contains("Exiting due to") && stderr.contains("warning(s)");
        if !has_fatal_errors && exiting_due_to_warnings {
            if config.debug {
                eprintln!(
                    "Verilator reported only warnings; treating as success.\n{}",
                    stderr
                );
            }
            return Ok(());
        }

        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        bail!(
            "Verilator lint failed (exit code {}):\nstdout:\n{}\nstderr:\n{}",
            code,
            stdout,
            stderr
        );
    }

    Ok(())
}
