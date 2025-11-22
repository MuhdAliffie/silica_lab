#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use tauri::Manager;
use verilog_compiler::{api::compile, config::CompilerConfig, runner::run_gtkwave};

// Global workspace path
static WORKSPACE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

// --- 0. Helper function to resolve paths ---
// Resolve paths relative to the workspace directory
fn resolve_workspace_path(relative_path: &str) -> PathBuf {
    let workspace = WORKSPACE_PATH.lock().unwrap();

    if let Some(workspace_path) = workspace.as_ref() {
        workspace_path.join(relative_path)
    } else {
        // Fallback to default backend path if no workspace is set
        let mut backend_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        backend_path.pop(); // Remove src-tauri
        backend_path.pop(); // Remove silica-lab-frontend
        backend_path.push("silica-lab");
        backend_path.push(relative_path);
        backend_path
    }
}

// --- Workspace Management ---
#[tauri::command]
fn set_workspace(path: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let workspace_path = PathBuf::from(&path);

    if !workspace_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !workspace_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut workspace = WORKSPACE_PATH.lock().unwrap();
    *workspace = Some(workspace_path.clone());
    drop(workspace);

    // Start watching the workspace
    start_watching(workspace_path, app_handle);

    Ok(format!("Workspace set to: {}", path))
}

fn start_watching(path: PathBuf, app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(_event) = res {
                    let _ = tx.send(());
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .ok();

        if let Some(ref mut w) = watcher {
            let _ = w.watch(&path, RecursiveMode::Recursive);

            // Debounce events - only emit after 500ms of no activity
            let mut last_event = std::time::Instant::now();
            loop {
                if rx.recv_timeout(Duration::from_millis(500)).is_ok() {
                    last_event = std::time::Instant::now();
                } else if last_event.elapsed() > Duration::from_millis(500) {
                    // Emit workspace change event to frontend
                    let _ = app_handle.emit_all("workspace-changed", ());
                    // Wait for next event
                    if rx.recv().is_err() {
                        break;
                    }
                    last_event = std::time::Instant::now();
                }
            }
        }
    });
}
#[tauri::command]
fn get_workspace() -> Result<Option<String>, String> {
    let workspace = WORKSPACE_PATH.lock().unwrap();
    Ok(workspace.as_ref().map(|p| p.to_string_lossy().to_string()))
}

// --- 1. Compile Verilog/SystemVerilog files ---
// Automatically detects language and uses appropriate compiler (iverilog/verilator)
#[tauri::command]
fn compile_verilog(files: Vec<String>, output_path: String) -> Result<String, String> {
    let file_paths: Vec<PathBuf> = files.iter().map(|f| resolve_workspace_path(f)).collect();
    let output = resolve_workspace_path(&output_path);

    let config = CompilerConfig::discover();

    compile(file_paths, &output, &config).map_err(|e| format!("Compilation failed: {}", e))?;

    Ok(format!("Compilation successful: {}", output_path))
}

// --- 2. Run VVP Simulation ---
// Executes the compiled simulation and generates VCD waveform file
#[tauri::command]
fn run_simulation(sim_path: String, vcd_path: String) -> Result<String, String> {
    let sim = resolve_workspace_path(&sim_path);
    let vcd = resolve_workspace_path(&vcd_path);

    // Ensure the VCD directory exists before simulation
    if let Some(parent) = vcd.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create VCD directory: {}", e))?;
    }

    // Run simulation from workspace directory so relative paths work
    let workspace = WORKSPACE_PATH.lock().unwrap();
    let workspace_dir = workspace
        .as_ref()
        .ok_or_else(|| "No workspace set".to_string())?;

    let config = CompilerConfig::discover();

    // Run vvp with workspace as working directory
    let default_vvp = PathBuf::from("vvp");
    let vvp_cmd = if let Some(path) = &config.vvp_path {
        path
    } else {
        &default_vvp
    };

    let output = std::process::Command::new(vvp_cmd)
        .arg(&sim)
        .current_dir(workspace_dir)
        .output()
        .map_err(|e| format!("Failed to execute vvp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Simulation failed: {}", stderr));
    }

    Ok(format!("Simulation completed. VCD at: {}", vcd_path))
}

// --- 3. Launch GTKWave (non-blocking) ---
// Opens waveform viewer without blocking the IDE
#[tauri::command]
fn spawn_gtkwave(vcd_path: String) -> Result<(), String> {
    let vcd = resolve_workspace_path(&vcd_path);

    let config = CompilerConfig::discover();

    run_gtkwave(&vcd, &config, true).map_err(|e| format!("Failed to launch GTKWave: {}", e))?;

    Ok(())
}

// --- 4. File System Operations ---

#[tauri::command]
fn scan_workspace_files() -> Result<Vec<String>, String> {
    let workspace = WORKSPACE_PATH.lock().unwrap();

    let workspace_path = workspace
        .as_ref()
        .ok_or_else(|| "No workspace set".to_string())?;

    let mut verilog_files = Vec::new();
    scan_directory_recursive(workspace_path, &workspace_path, &mut verilog_files)
        .map_err(|e| format!("Failed to scan workspace: {}", e))?;

    Ok(verilog_files)
}

fn scan_directory_recursive(
    dir: &Path,
    root: &Path,
    files: &mut Vec<String>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                scan_directory_recursive(&path, root, files)?;
            } else {
                // Include all files
                if let Ok(relative) = path.strip_prefix(root) {
                    files.push(relative.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn read_directory(path: String) -> Result<Vec<String>, String> {
    let full_path = resolve_workspace_path(&path);
    let entries =
        fs::read_dir(full_path).map_err(|e| format!("Failed to read directory: {}", e))?;

    let files: Vec<String> = entries
        .filter_map(|entry| {
            entry
                .ok()
                .and_then(|e| e.file_name().to_str().map(|s| s.to_string()))
        })
        .collect();

    Ok(files)
}

#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    let full_path = resolve_workspace_path(&path);
    fs::read_to_string(full_path).map_err(|e| format!("Error reading file: {}", e))
}

#[tauri::command]
fn save_file(path: String, content: String) -> Result<(), String> {
    let full_path = resolve_workspace_path(&path);

    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(full_path, content).map_err(|e| format!("Error writing file: {}", e))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_workspace,
            get_workspace,
            scan_workspace_files,
            compile_verilog,
            run_simulation,
            spawn_gtkwave,
            read_directory,
            read_file,
            save_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
