<script>
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/api/dialog";
  import { listen } from "@tauri-apps/api/event";
  import {
    FileCode,
    Play,
    Activity,
    Monitor,
    Layers,
    FolderOpen,
  } from "lucide-svelte";
  import MonacoEditor from "./lib/components/MonacoEditor.svelte";
  import {
    terminalOutput,
    currentFile,
    activeTool,
    dirtyFiles,
  } from "./lib/stores";
  import { ICTools, FileManager } from "./lib/backend";

  let workspacePath = null;
  let files = [];
  let verilogFiles = []; // Track .v/.sv files for compilation
  let vcdFiles = []; // Track .vcd files
  let unlistenWorkspaceChanged = null;
  let openTabs = [];
  let activeTabIndex = 0;
  let showVcdSelector = false;
  let selectedVcdForViewer = null;
  let showBuildSelector = false;
  let selectedVerilogFiles = [];
  let selectedTestbench = null;
  let terminalHeight = 192; // 48 * 4 = 192px (12rem)
  let isResizing = false;
  let startY = 0;
  let startHeight = 0;
  let sidebarWidth = 256; // 16rem (w-64)
  let isResizingSidebar = false;
  let startX = 0;
  let startWidth = 0;

  function startResize(e) {
    isResizing = true;
    startY = e.clientY;
    startHeight = terminalHeight;
    document.body.style.cursor = "ns-resize";
  }

  function handleResize(e) {
    if (!isResizing) return;
    const delta = startY - e.clientY;
    const newHeight = Math.max(100, Math.min(600, startHeight + delta));
    terminalHeight = newHeight;
  }

  function stopResize() {
    isResizing = false;
    document.body.style.cursor = "default";
  }

  function startResizeSidebar(e) {
    isResizingSidebar = true;
    startX = e.clientX;
    startWidth = sidebarWidth;
    document.body.style.cursor = "ew-resize";
  }

  function handleResizeSidebar(e) {
    if (!isResizingSidebar) return;
    const delta = e.clientX - startX;
    const newWidth = Math.max(200, Math.min(600, startWidth + delta));
    sidebarWidth = newWidth;
  }

  function stopResizeSidebar() {
    isResizingSidebar = false;
    document.body.style.cursor = "default";
  }

  onMount(async () => {
    window.addEventListener("mousemove", handleResize);
    window.addEventListener("mouseup", stopResize);
    window.addEventListener("mousemove", handleResizeSidebar);
    window.addEventListener("mouseup", stopResizeSidebar);

    // Check if workspace is already set
    const workspace = await FileManager.getWorkspace();
    if (workspace) {
      workspacePath = workspace;
      await loadWorkspaceFiles();
    }

    // Listen for workspace file changes
    unlistenWorkspaceChanged = await listen("workspace-changed", async () => {
      await loadWorkspaceFiles();
    });
  });

  onDestroy(() => {
    window.removeEventListener("mousemove", handleResize);
    window.removeEventListener("mouseup", stopResize);
    window.removeEventListener("mousemove", handleResizeSidebar);
    window.removeEventListener("mouseup", stopResizeSidebar);
    // Cleanup event listener
    if (unlistenWorkspaceChanged) {
      unlistenWorkspaceChanged();
    }
  });

  async function openFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Open Workspace Folder",
    });

    if (selected) {
      try {
        await FileManager.setWorkspace(selected);
        workspacePath = selected;
        await loadWorkspaceFiles();
      } catch (e) {
        terminalOutput.update((lines) => [
          ...lines,
          { type: "error", text: `Failed to open folder: ${e}` },
        ]);
      }
    }
  }

  // Build tree structure from flat file paths
  function buildTree(filePaths) {
    const root = { name: "", children: {}, files: [] };

    filePaths.forEach((path) => {
      const parts = path.split("/");
      let current = root;

      // Navigate/create folder structure
      for (let i = 0; i < parts.length - 1; i++) {
        const folder = parts[i];
        if (!current.children[folder]) {
          current.children[folder] = {
            name: folder,
            children: {},
            files: [],
            expanded: false,
          };
        }
        current = current.children[folder];
      }

      // Add file
      const fileName = parts[parts.length - 1];
      const ext = fileName.split(".").pop();
      current.files.push({
        name: fileName,
        path,
        type: ext === "vcd" ? "wave" : "file",
      });
    });

    return root;
  }

  let fileTree = { children: {}, files: [] };

  async function loadWorkspaceFiles() {
    try {
      const fileList = await FileManager.scanWorkspaceFiles();
      files = fileList.map((path) => {
        const name = path.split("/").pop();
        const ext = name.split(".").pop();
        return {
          name,
          path,
          type: ext === "vcd" ? "wave" : "file",
        };
      });

      // Build tree structure
      fileTree = buildTree(fileList);

      // Separate verilog source files for compilation
      verilogFiles = fileList.filter((p) => {
        const ext = p.split(".").pop();
        return ext === "v" || ext === "sv" || ext === "vh" || ext === "svh";
      });
      vcdFiles = fileList.filter((p) => p.endsWith(".vcd"));

      terminalOutput.update((lines) => [
        ...lines,
        {
          type: "success",
          text: `Loaded ${fileList.length} files from workspace`,
        },
      ]);
    } catch (e) {
      terminalOutput.update((lines) => [
        ...lines,
        { type: "error", text: `Failed to scan workspace: ${e}` },
      ]);
    }
  }

  function toggleFolder(folder) {
    folder.expanded = !folder.expanded;
    fileTree = fileTree; // Trigger reactivity
  }

  function openBuildSelector() {
    if (verilogFiles.length === 0) {
      terminalOutput.update((lines) => [
        ...lines,
        { type: "error", text: "No Verilog files found in workspace" },
      ]);
      return;
    }

    // Reset selections
    selectedVerilogFiles = [...verilogFiles];
    selectedTestbench = null;
    showBuildSelector = true;
  }

  function toggleVerilogFile(filePath) {
    if (selectedVerilogFiles.includes(filePath)) {
      selectedVerilogFiles = selectedVerilogFiles.filter((f) => f !== filePath);
    } else {
      selectedVerilogFiles = [...selectedVerilogFiles, filePath];
    }
  }

  async function executeBuildAndSimulate() {
    if (selectedVerilogFiles.length === 0) {
      terminalOutput.update((lines) => [
        ...lines,
        { type: "error", text: "Please select at least one Verilog file" },
      ]);
      return;
    }

    showBuildSelector = false;

    const outputPath = "build/sim.out";
    const vcdPath = "build/wave.vcd";

    try {
      await ICTools.buildAndSimulate(selectedVerilogFiles, outputPath, vcdPath);
      // Small delay to ensure VCD file is fully written
      await new Promise((resolve) => setTimeout(resolve, 500));
      // Refresh file list to show newly generated files
      await loadWorkspaceFiles();
    } catch (e) {
      // Error already logged by ICTools
    }
  }

  async function buildAndSimulateWorkspace() {
    if (verilogFiles.length === 0) {
      terminalOutput.update((lines) => [
        ...lines,
        { type: "error", text: "No Verilog files found in workspace" },
      ]);
      return;
    }

    const outputPath = "build/sim.out";
    const vcdPath = "build/wave.vcd";

    try {
      await ICTools.buildAndSimulate(verilogFiles, outputPath, vcdPath);
      // Small delay to ensure VCD file is fully written
      await new Promise((resolve) => setTimeout(resolve, 500));
      // Refresh file list to show newly generated files
      await loadWorkspaceFiles();
    } catch (e) {
      // Error already logged by ICTools
    }
  }

  function openWaveformViewer() {
    if (vcdFiles.length === 0) {
      terminalOutput.update((lines) => [
        ...lines,
        { type: "error", text: "No VCD files found in workspace" },
      ]);
      return;
    }

    // Always show selector to let user choose which VCD file to open
    showVcdSelector = true;
  }

  function selectVcdFile(vcdPath) {
    selectedVcdForViewer = vcdPath;
    showVcdSelector = false;
    ICTools.openGtkwave(vcdPath);
  }

  async function openFile(file) {
    // Check if file is already open in a tab
    const existingTabIndex = openTabs.findIndex(
      (tab) => tab.path === file.path,
    );
    if (existingTabIndex !== -1) {
      activeTabIndex = existingTabIndex;
      currentFile.set(openTabs[activeTabIndex]);
      return;
    }

    // Check if it's a binary file that should be read as text anyway
    const ext = file.name.split(".").pop();
    const textExtensions = ["v", "sv", "vh", "svh", "vcd", "out", "txt", "log"];
    const isBinary = !textExtensions.includes(ext);

    try {
      let content;
      if (isBinary) {
        content = `// Binary file: ${file.name}\n// Path: ${file.path}\n// Cannot display binary content`;
      } else {
        content = await FileManager.readFile(file.path);
      }

      const newTab = { ...file, content };
      openTabs = [...openTabs, newTab];
      activeTabIndex = openTabs.length - 1;
      currentFile.set(newTab);
    } catch (e) {
      terminalOutput.update((lines) => [
        ...lines,
        {
          type: "error",
          text: `Failed to read ${file.path}: ${e}`,
        },
      ]);
      const errorTab = {
        ...file,
        content: `// Error reading file: ${e}\n// Path: ${file.path}`,
      };
      openTabs = [...openTabs, errorTab];
      activeTabIndex = openTabs.length - 1;
      currentFile.set(errorTab);
    }
  }

  function closeTab(index, event) {
    event.stopPropagation();
    openTabs = openTabs.filter((_, i) => i !== index);

    if (openTabs.length === 0) {
      currentFile.set({ name: "", path: "", content: "" });
      activeTabIndex = 0;
    } else if (activeTabIndex >= openTabs.length) {
      activeTabIndex = openTabs.length - 1;
      currentFile.set(openTabs[activeTabIndex]);
    } else if (activeTabIndex === index) {
      currentFile.set(openTabs[activeTabIndex]);
    }
  }

  function switchTab(index) {
    activeTabIndex = index;
    currentFile.set(openTabs[index]);
  }

  function handleContentChanged(event) {
    const filePath = event.detail.path;
    dirtyFiles.update((files) => {
      files.add(filePath);
      return files;
    });
  }

  function handleFileSaved(event) {
    const filePath = event.detail.path;
    dirtyFiles.update((files) => {
      files.delete(filePath);
      return files;
    });
    terminalOutput.update((lines) => [
      ...lines,
      { type: "success", text: `Saved: ${filePath}` },
    ]);
  }
</script>

<div
  class="flex h-screen w-screen bg-[#1e1e1e] text-gray-300 overflow-hidden font-sans"
>
  <div
    class="w-12 flex flex-col items-center py-4 bg-[#333333] border-r border-gray-700 z-10"
  >
    <button
      class="p-2 mb-2 hover:text-white {$activeTool === 'explorer'
        ? 'text-white border-l-2 border-blue-500'
        : 'text-gray-500'}"
      on:click={() => activeTool.set("explorer")}
    >
      <FileCode size={24} />
    </button>
  </div>

  <div
    class="bg-[#252526] flex flex-col border-r border-gray-700 relative"
    style="display: {$activeTool === 'explorer'
      ? 'flex'
      : 'none'}; width: {sidebarWidth}px"
  >
    <div class="p-3 flex items-center justify-between border-b border-gray-700">
      <div class="text-xs font-bold uppercase tracking-wider">
        Silica Explorer
      </div>
      <button
        class="p-1 hover:bg-[#2a2d2e] rounded"
        on:click={openFolder}
        title="Open Folder"
      >
        <FolderOpen size={16} />
      </button>
    </div>
    {#if workspacePath}
      <div
        class="px-3 py-2 text-xs text-gray-500 border-b border-gray-700 truncate"
        title={workspacePath}
      >
        {workspacePath}
      </div>
    {/if}
    <div class="flex-1 overflow-y-auto">
      {#each Object.values(fileTree.children) as folder}
        <div>
          <button
            class="w-full px-2 py-1 flex items-center text-sm hover:bg-[#2a2d2e] text-left"
            on:click={() => toggleFolder(folder)}
          >
            <span class="mr-1">{folder.expanded ? "▼" : "▶"}</span>
            <span>{folder.name}</span>
          </button>
          {#if folder.expanded}
            <div class="ml-4">
              {#each folder.files as file}
                <button
                  class="w-full px-2 py-1 flex items-center text-sm hover:bg-[#2a2d2e] {file.path ===
                  $currentFile.path
                    ? 'bg-[#37373d] text-white'
                    : ''} text-left"
                  on:click={() => openFile(file)}
                >
                  <span class="ml-2">{file.name}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
      {#each fileTree.files as file}
        <button
          class="w-full px-2 py-1 flex items-center text-sm hover:bg-[#2a2d2e] {file.path ===
          $currentFile.path
            ? 'bg-[#37373d] text-white'
            : ''} text-left"
          on:click={() => openFile(file)}
        >
          <span class="ml-2">{file.name}</span>
        </button>
      {/each}
    </div>

    <!-- Resize Handle for Sidebar -->
    <button
      type="button"
      aria-label="Resize sidebar"
      class="absolute top-0 right-0 w-1 h-full bg-transparent hover:bg-blue-500 cursor-ew-resize border-0"
      on:mousedown={startResizeSidebar}
    ></button>
  </div>

  <div class="flex-1 flex flex-col min-w-0">
    <div
      class="h-10 bg-[#1e1e1e] flex items-center border-b border-gray-700 px-4 justify-between"
    >
      <div class="flex gap-2">
        <button
          class="flex items-center gap-1 px-3 py-1 bg-green-700 hover:bg-green-600 text-white text-xs rounded"
          on:click={openBuildSelector}
          disabled={verilogFiles.length === 0}
          title={verilogFiles.length === 0
            ? "No Verilog files in workspace"
            : `Build ${verilogFiles.length} file(s)`}
        >
          <Play size={12} /> Build & Sim
        </button>
        <button
          class="flex items-center gap-1 px-3 py-1 bg-blue-700 hover:bg-blue-600 text-white text-xs rounded"
          on:click={openWaveformViewer}
        >
          <Activity size={12} /> Wave
        </button>
      </div>
    </div>

    <div class="flex-1 flex flex-col min-h-0 relative">
      <!-- Tab Bar -->
      {#if openTabs.length > 0}
        <div
          class="flex bg-[#252526] border-b border-gray-700 overflow-x-auto z-20 relative"
        >
          {#each openTabs as tab, index}
            <button
              class="flex items-center gap-2 px-3 py-0.5 text-sm border-r border-gray-700 {index ===
              activeTabIndex
                ? 'bg-[#1e1e1e] text-white'
                : 'bg-[#2d2d30] text-gray-400 hover:text-white'}"
              on:click={() => switchTab(index)}
            >
              <span class="flex items-center gap-1 truncate max-w-[150px]">
                {#if $dirtyFiles.has(tab.path)}
                  <span class="text-white text-xs leading-none">●</span>
                {/if}
                {tab.name}
              </span>
              <button
                class="hover:bg-gray-600 rounded px-1.5 py-0.5 text-base"
                on:click={(e) => closeTab(index, e)}
              >
                ×
              </button>
            </button>
          {/each}
        </div>
      {/if}

      <div class="flex-1 relative min-h-0">
        <MonacoEditor
          on:contentChanged={handleContentChanged}
          on:fileSaved={handleFileSaved}
        />
      </div>

      <!-- Resizable Terminal Overlay -->
      <div
        class="absolute bottom-0 left-0 right-0 bg-[#1e1e1e] border-t border-gray-700 flex flex-col z-10"
        style="height: {terminalHeight}px"
      >
        <!-- Resize Handle -->
        <button
          type="button"
          aria-label="Resize terminal"
          class="h-1 w-full bg-[#252526] hover:bg-blue-500 cursor-ns-resize border-0"
          on:mousedown={startResize}
        ></button>
        <div
          class="flex gap-4 px-4 py-1 border-b border-gray-700 text-xs uppercase text-white"
        >
          Output
        </div>
        <div
          class="flex-1 p-2 overflow-y-auto font-mono text-xs bg-black text-gray-300"
        >
          {#each $terminalOutput as line}
            <div
              class={line.type === "error"
                ? "text-red-400"
                : line.type === "success"
                  ? "text-green-400"
                  : "text-gray-300"}
            >
              > {line.text}
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- Build File Selector Modal -->
    {#if showBuildSelector}
      <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      >
        <div
          class="bg-[#252526] p-6 rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] flex flex-col"
        >
          <h3 class="text-white text-lg mb-4">
            Select Files to Build & Simulate
          </h3>

          <div class="flex-1 overflow-y-auto mb-4">
            <div class="mb-6">
              <h4 class="text-white text-sm font-semibold mb-2">
                Verilog Files
              </h4>
              <div class="space-y-1">
                {#each verilogFiles as vFile}
                  <label
                    class="flex items-center px-3 py-2 bg-[#2d2d30] hover:bg-[#37373d] rounded text-sm cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      class="mr-3"
                      checked={selectedVerilogFiles.includes(vFile)}
                      on:change={() => toggleVerilogFile(vFile)}
                    />
                    <span class="text-gray-300">{vFile}</span>
                  </label>
                {/each}
              </div>
            </div>

            <div>
              <h4 class="text-white text-sm font-semibold mb-2">
                Testbench (Optional)
              </h4>
              <div class="space-y-1">
                {#each verilogFiles.filter((f) => f.includes("tb") || f.includes("test")) as tbFile}
                  <label
                    class="flex items-center px-3 py-2 bg-[#2d2d30] hover:bg-[#37373d] rounded text-sm cursor-pointer"
                  >
                    <input
                      type="radio"
                      name="testbench"
                      class="mr-3"
                      checked={selectedTestbench === tbFile}
                      on:change={() => (selectedTestbench = tbFile)}
                    />
                    <span class="text-gray-300">{tbFile}</span>
                  </label>
                {/each}
                {#if verilogFiles.filter((f) => f.includes("tb") || f.includes("test")).length === 0}
                  <p class="text-gray-500 text-xs italic px-3 py-2">
                    No testbench files detected (files with 'tb' or 'test' in
                    name)
                  </p>
                {/if}
              </div>
            </div>
          </div>

          <div class="flex gap-2">
            <button
              class="flex-1 px-4 py-2 bg-green-700 hover:bg-green-600 rounded text-sm text-white"
              on:click={executeBuildAndSimulate}
            >
              Build & Simulate ({selectedVerilogFiles.length} files)
            </button>
            <button
              class="px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded text-sm text-white"
              on:click={() => (showBuildSelector = false)}
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    {/if}

    <!-- VCD File Selector Modal -->
    {#if showVcdSelector}
      <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      >
        <div class="bg-[#252526] p-4 rounded-lg shadow-xl max-w-md w-full">
          <h3 class="text-white text-lg mb-4">Select VCD File to View</h3>
          <div class="space-y-2 max-h-96 overflow-y-auto">
            {#each vcdFiles as vcdPath}
              <button
                class="w-full text-left px-3 py-2 bg-[#2d2d30] hover:bg-[#37373d] rounded text-sm"
                on:click={() => selectVcdFile(vcdPath)}
              >
                {vcdPath}
              </button>
            {/each}
          </div>
          <button
            class="mt-4 px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded text-sm w-full"
            on:click={() => (showVcdSelector = false)}
          >
            Cancel
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>
