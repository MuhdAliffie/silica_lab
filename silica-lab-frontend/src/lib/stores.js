import { writable } from 'svelte/store';

export const currentFile = writable({
    name: 'top.v',
    path: '/src/top.v',
    content: '// Silica Lab: Select a file...'
});

export const terminalOutput = writable([
    { type: 'info', text: 'Silica Lab initialized.' }
]);

export const activeTool = writable('explorer');

export const dirtyFiles = writable(new Set());
