use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerilogLang {
    Verilog,
    SystemVerilog,
}

pub fn detect_language(file: &Path) -> VerilogLang {
    match file.extension().and_then(|e| e.to_str()) {
        Some("v") => VerilogLang::Verilog,
        Some("sv") => VerilogLang::SystemVerilog,
        _ => VerilogLang::Verilog, // default fallback
    }
}
