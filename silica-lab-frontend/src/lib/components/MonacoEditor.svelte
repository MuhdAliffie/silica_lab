<script>
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { currentFile } from '../stores';
  import { FileManager } from '../backend';

  let editorContainer;
  let editor;

  $: if (editor && $currentFile) {
      const model = editor.getModel();
      if (model && model.getValue() !== $currentFile.content) {
          editor.setValue($currentFile.content);
          const lang = $currentFile.name.endsWith('.v') ? 'verilog' : 'plaintext';
          monaco.editor.setModelLanguage(model, lang);
      }
  }

  onMount(() => {
    editor = monaco.editor.create(editorContainer, {
      value: $currentFile.content,
      language: 'verilog',
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: true },
      fontSize: 14
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, async () => {
        await FileManager.saveFile($currentFile.path, editor.getValue());
    });
  });

  onDestroy(() => { editor?.dispose(); });
</script>
<div class="w-full h-full overflow-hidden" bind:this={editorContainer}></div>
