import { invoke } from '@tauri-apps/api/tauri';
import { terminalOutput } from './stores';

const log = (text, type = 'info') => {
    terminalOutput.update(lines => [...lines, { type, text }]);
};

export const FileManager = {
    async setWorkspace(path) {
        try {
            const res = await invoke('set_workspace', { path });
            log(res, 'success');
            return res;
        } catch (err) {
            log(`Workspace Error: ${err}`, 'error');
            throw err;
        }
    },
    async getWorkspace() {
        return await invoke('get_workspace');
    },
    async scanWorkspaceFiles() {
        return await invoke('scan_workspace_files');
    },
    async readDir(path) {
        return await invoke('read_directory', { path });
    },
    async readFile(path) {
        return await invoke('read_file', { path });
    },
    async saveFile(path, content) {
        await invoke('save_file', { path, content });
        log(`Saved ${path}`, 'success');
    }
};

export const ICTools = {
    async compileVerilog(files, outputPath) {
        log(`Compiling ${files.length} file(s)...`, 'info');
        try {
            const res = await invoke('compile_verilog', { files, outputPath });
            log(res, 'success');
            return outputPath;
        } catch (err) {
            log(`Compilation Error: ${err}`, 'error');
            throw err;
        }
    },
    async runSimulation(simPath, vcdPath) {
        log(`Running simulation...`, 'info');
        try {
            const res = await invoke('run_simulation', { simPath, vcdPath });
            log(res, 'success');
            return vcdPath;
        } catch (err) {
            log(`Simulation Error: ${err}`, 'error');
            throw err;
        }
    },
    async openGtkwave(vcdPath) {
        log('Launching GTKWave...', 'info');
        try {
            await invoke('spawn_gtkwave', { vcdPath });
            log('GTKWave launched successfully', 'success');
        } catch (err) {
            log(`GTKWave Error: ${err}`, 'error');
            throw err;
        }
    },
    // Convenience method: compile -> simulate -> view in one workflow
    async buildAndSimulate(files, outputPath, vcdPath) {
        await this.compileVerilog(files, outputPath);
        await this.runSimulation(outputPath, vcdPath);
        return vcdPath;
    },
    async runOpenROAD(tclScript) {
        log('Starting OpenROAD...', 'warning');
        // Placeholder for future implementation
    }
};
