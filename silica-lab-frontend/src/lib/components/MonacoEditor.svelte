<script>
  import { onMount, onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";
  import * as monaco from "monaco-editor";
  import { currentFile, dirtyFiles } from "../stores";
  import { FileManager } from "../backend";

  const dispatch = createEventDispatcher();
  let editorContainer;
  let editor;
  let currentFilePath = null;
  let isLoadingFile = false;

  $: if (editor && $currentFile) {
    const model = editor.getModel();
    if (model && model.getValue() !== $currentFile.content) {
      isLoadingFile = true;
      editor.setValue($currentFile.content);
      const ext = $currentFile.name.split(".").pop();
      const lang = ext === "v" || ext === "sv" ? "verilog" : "plaintext";
      monaco.editor.setModelLanguage(model, lang);
      currentFilePath = $currentFile.path;

      // Clear dirty state for newly loaded file
      dirtyFiles.update((files) => {
        files.delete(currentFilePath);
        return files;
      });

      // Reset flag after a brief delay to allow Monaco to settle
      setTimeout(() => {
        isLoadingFile = false;
      }, 100);
    }
  }

  onMount(() => {
    isLoadingFile = true;
    editor = monaco.editor.create(editorContainer, {
      value: $currentFile.content,
      language: "verilog",
      theme: "vs-dark",
      automaticLayout: true,
      minimap: { enabled: true },
      fontSize: 14,
    });

    currentFilePath = $currentFile.path;

    // Reset flag after initial load
    setTimeout(() => {
      isLoadingFile = false;
    }, 100);

    // Track content changes (but not during file loads)
    editor.onDidChangeModelContent(() => {
      if (currentFilePath && !isLoadingFile) {
        dispatch("contentChanged", { path: currentFilePath });
      }
    });

    // Save command (Ctrl+S / Cmd+S)
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, async () => {
      const content = editor.getValue();
      await FileManager.saveFile($currentFile.path, content);
      dispatch("fileSaved", { path: $currentFile.path });
    });
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div class="w-full h-full overflow-hidden" bind:this={editorContainer}></div>
