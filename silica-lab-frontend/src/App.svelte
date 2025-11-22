<script>
  import { FileCode, Play, Activity, Monitor, Layers } from 'lucide-svelte';
  import MonacoEditor from './lib/components/MonacoEditor.svelte';
  import { terminalOutput, currentFile, activeTool } from './lib/stores';
  import { ICTools, FileManager } from './lib/backend';

  // MOCK DATA: Replace with actual FileManager.readDir() call in onMount
  let files = [
      { name: 'cpu_core.v', path: '/src/cpu_core.v', type: 'file' },
      { name: 'cpu_tb.v', path: '/src/cpu_tb.v', type: 'file' },
      { name: 'dump.vcd', path: '/sim/dump.vcd', type: 'wave' }
  ];

  async function openFile(file) {
      if(file.type === 'wave') {
          ICTools.openGtkwave(file.path);
          return;
      }
      try {
        const content = await FileManager.readFile(file.path);
        currentFile.set({ ...file, content });
      } catch (e) {
        // If backend fails (e.g. file doesn't exist on disk yet), load mock
        currentFile.set({ ...file, content: "// Backend not connected or file not found." });
      }
  }
</script>

<div class="flex h-screen w-screen bg-[#1e1e1e] text-gray-300 overflow-hidden font-sans">
  <div class="w-12 flex flex-col items-center py-4 bg-[#333333] border-r border-gray-700 z-10">
    <button class="p-2 mb-2 hover:text-white {$activeTool === 'explorer' ? 'text-white border-l-2 border-blue-500' : 'text-gray-500'}" on:click={() => activeTool.set('explorer')}>
      <FileCode size={24} />
    </button>
  </div>

  <div class="w-64 bg-[#252526] flex flex-col border-r border-gray-700" style="display: {$activeTool === 'explorer' ? 'flex' : 'none'}">
    <div class="p-3 text-xs font-bold uppercase tracking-wider">Silica Explorer</div>
    <div class="flex-1 overflow-y-auto">
        {#each files as file}
            <div class="px-4 py-1 cursor-pointer flex items-center text-sm hover:bg-[#2a2d2e] {file.path === $currentFile.path ? 'bg-[#37373d] text-white' : ''}" on:click={() => openFile(file)}>
                <span class="ml-2">{file.name}</span>
            </div>
        {/each}
    </div>
  </div>

  <div class="flex-1 flex flex-col min-w-0">
    <div class="h-10 bg-[#1e1e1e] flex items-center border-b border-gray-700 px-4 justify-between">
        <div class="text-sm italic text-gray-400">{$currentFile.name}</div>
        <div class="flex gap-2">
            <button class="flex items-center gap-1 px-3 py-1 bg-green-700 hover:bg-green-600 text-white text-xs rounded" on:click={() => ICTools.runIverilog($currentFile.path)}>
                <Play size={12} /> Build
            </button>
            <button class="flex items-center gap-1 px-3 py-1 bg-blue-700 hover:bg-blue-600 text-white text-xs rounded" on:click={() => ICTools.openGtkwave('dump.vcd')}>
                <Activity size={12} /> Wave
            </button>
        </div>
    </div>
    <div class="flex-1 relative"><MonacoEditor /></div>
    
    <div class="h-48 bg-[#1e1e1e] border-t border-gray-700 flex flex-col">
        <div class="flex gap-4 px-4 py-1 border-b border-gray-700 text-xs uppercase text-white">Output</div>
        <div class="flex-1 p-2 overflow-y-auto font-mono text-xs bg-black text-gray-300">
            {#each $terminalOutput as line}
                <div class="{line.type === 'error' ? 'text-red-400' : line.type === 'success' ? 'text-green-400' : 'text-gray-300'}">> {line.text}</div>
            {/each}
        </div>
    </div>
  </div>
</div>
