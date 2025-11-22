#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::process::Command;

// --- 1. Detached GTKWave Process ---
// This is the critical part for your requirement.
// Using spawn() ensures the IDE doesn't freeze while GTKWave runs.
#[tauri::command]
fn spawn_gtkwave(path: String) -> Result<(), String> {
    Command::new("gtkwave")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to launch GTKWave: {}", e))?;
    Ok(())
}

// --- 2. Iverilog Compilation (Blocking) ---
#[tauri::command]
fn run_iverilog(file_path: String) -> Result<String, String> {
    let output = Command::new("iverilog")
        .arg("-o")
        .arg("out.vvp")
        .arg(&file_path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok("Compilation successful.".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// --- 3. File System Placeholders ---
// You can replace these with your existing backend logic
#[tauri::command]
fn read_directory(_path: String) -> Vec<String> {
    // Mock return
    vec!["cpu.v".to_string(), "tb.v".to_string()]
}

#[tauri::command]
fn read_file(path: String) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| "// Error reading file".to_string())
}

#[tauri::command]
fn save_file(path: String, content: String) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            spawn_gtkwave,
            run_iverilog,
            read_directory,
            read_file,
            save_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
