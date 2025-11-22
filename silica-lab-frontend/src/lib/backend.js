import { invoke } from '@tauri-apps/api/tauri';
import { terminalOutput } from './stores';

const log = (text, type = 'info') => {
    terminalOutput.update(lines => [...lines, { type, text }]);
};

export const FileManager = {
    async readDir(path) {
        // Calls Rust 'read_directory'
        return await invoke('read_directory', { path });
    },
    async readFile(path) {
        // Calls Rust 'read_file'
        return await invoke('read_file', { path });
    },
    async saveFile(path, content) {
        await invoke('save_file', { path, content });
        log(`Saved ${path}`, 'success');
    }
};

export const ICTools = {
    async runIverilog(filePath) {
        log(`Building ${filePath}...`, 'info');
        try {
            const res = await invoke('run_iverilog', { filePath });
            log(res, 'success');
        } catch (err) {
            log(`Error: ${err}`, 'error');
        }
    },
    async openGtkwave(wavePath) {
        log('Launching GTKWave...', 'info');
        // This calls the DETACHED process in Rust
        await invoke('spawn_gtkwave', { path: wavePath });
    },
    async runOpenROAD(tclScript) {
        log('Starting OpenROAD...', 'warning');
        // Placeholder for future implementation
    }
};
