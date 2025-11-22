use std::path::Path; // Added Path import
use std::path::PathBuf;
use verilog_compiler::runner::{run_gtkwave, run_vvp};
use verilog_compiler::{api::compile, config::CompilerConfig}; // Added runner imports

fn main() {
    // Test 1: Icarus Verilog compile of Verilog sources -> produces a vvp-ready sim binary

    // Declaring paths for Verilog source files
    let verilog_files = vec![
        PathBuf::from("examples/verilog_ex/top.v"),
        PathBuf::from("examples/verilog_ex/tb.v"),
    ];
    let output = PathBuf::from("build/sim.out");
    let config = CompilerConfig {
        debug: true,
        ..Default::default()
    };

    compile(verilog_files, &output, &config).unwrap();
    println!(
        "Icarus compilation finished successfully: {}",
        output.display()
    );

    // Run the simulation with vvp to produce the VCD
    let vcd_path = Path::new("build/wave.vcd");
    run_vvp(&output, vcd_path, &config).unwrap();
    println!("Simulation finished; VCD at {}", vcd_path.display());

    // Optionally launch GTKWave to view the waveform (non-blocking)
    run_gtkwave(vcd_path, &config, true).unwrap();

    // Test 2: Verilator lint of SystemVerilog testbench + referenced top module
    // Provide all referenced sources so Verilator can resolve modules.
    let sv_files = vec![
        PathBuf::from("examples/systemverilog_ex/tb.sv"),
        PathBuf::from("examples/verilog_ex/top.v"),
    ];
    verilog_compiler::compiler::verilator::compile_verilator(&sv_files, &config).unwrap();
    println!("Verilator lint finished successfully!");
}
