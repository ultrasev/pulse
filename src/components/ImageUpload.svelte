<script lang="ts">
    import { listen } from '@tauri-apps/api/event';
    import { open } from '@tauri-apps/plugin-shell';
    import { onMount } from 'svelte';

    type UploadResult = {
        success: boolean;
        url?: string;
        filename?: string;
        size?: string;
        duration?: string;
        error?: string;
    };

    let uploadResult: UploadResult | null = null;
    let isUploading = false;
    let unsubscribe: (() => void) | null = null;
    let copyClicked = false;
    let copyTimeout: ReturnType<typeof setTimeout> | null = null;

    onMount(() => {
        // Listen for upload result event from Rust
        const unlisten = listen<UploadResult>('upload-result', (event) => {
            uploadResult = event.payload;
            isUploading = false;
        });

        // Listen for upload start
        const unlistenUpload = listen('upload-start', () => {
            isUploading = true;
        });

        unsubscribe = () => {
            unlisten.then(fn => fn());
            unlistenUpload.then(fn => fn());
        };
    });

    async function copyToClipboard(text: string) {
        await navigator.clipboard.writeText(text);
        copyClicked = true;
        if (copyTimeout) clearTimeout(copyTimeout);
        copyTimeout = setTimeout(() => {
            copyClicked = false;
        }, 2000);
    }

    async function openUrl(url: string) {
        await open(url);
    }
</script>

<div class="h-full flex flex-col p-4">
    <header class="mb-4">
        <h2 class="text-xl font-bold">Image Upload</h2>
        <p class="text-sm text-gray-500 dark:text-gray-400">
            Press <kbd class="px-2 py-1 bg-gray-200 dark:bg-gray-700 rounded">Shift+Cmd+U</kbd> to upload image from clipboard
        </p>
    </header>

    {#if isUploading}
        <div class="flex-1 flex items-center justify-center">
            <div class="text-center">
                <svg class="animate-spin h-12 w-12 text-blue-500 mx-auto mb-4" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                <p class="text-gray-600 dark:text-gray-400">Uploading image...</p>
            </div>
        </div>
    {:else if uploadResult}
        <div class="flex-1 flex items-center justify-center">
            {#if uploadResult.success}
                <div class="w-full max-w-md p-6 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
                    <div class="flex items-center gap-3 mb-4">
                        <span class="text-3xl">✓</span>
                        <h3 class="text-lg font-bold text-green-800 dark:text-green-200">Upload Successful!</h3>
                    </div>

                    <div class="space-y-3">
                        <div>
                            <span class="text-sm text-gray-500 dark:text-gray-400">Filename</span>
                            <p class="font-medium">{uploadResult.filename}</p>
                        </div>

                        <div>
                            <span class="text-sm text-gray-500 dark:text-gray-400">Size</span>
                            <p class="font-medium">{uploadResult.size}</p>
                        </div>

                        <div>
                            <span class="text-sm text-gray-500 dark:text-gray-400">URL</span>
                            <div class="flex gap-2 mt-1">
                                <code class="flex-1 px-3 py-2 bg-white dark:bg-gray-800 rounded border text-sm overflow-x-auto">
                                    {uploadResult.url}
                                </code>
                                <button
                                    on:click={() => copyToClipboard(uploadResult.url!)}
                                    class="px-4 py-2 {copyClicked ? 'bg-green-500 hover:bg-green-600' : 'bg-blue-500 hover:bg-blue-600'} text-white rounded transition-colors min-w-[70px]"
                                >
                                    {copyClicked ? 'Copied!' : 'Copy'}
                                </button>
                                <button
                                    on:click={() => openUrl(uploadResult.url!)}
                                    class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
                                >
                                    Open
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            {:else}
                <div class="w-full max-w-md p-6 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
                    <div class="flex items-center gap-3 mb-4">
                        <span class="text-3xl">✕</span>
                        <h3 class="text-lg font-bold text-red-800 dark:text-red-200">Upload Failed</h3>
                    </div>
                    <p class="text-gray-700 dark:text-gray-300">{uploadResult.error}</p>
                </div>
            {/if}
        </div>
    {:else}
        <div class="flex-1 flex items-center justify-center">
            <div class="text-center text-gray-400 dark:text-gray-600">
                <svg class="w-24 h-24 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
                </svg>
                <p class="text-lg">No image uploaded yet</p>
                <p class="text-sm mt-2">Press Shift+Cmd+U to upload from clipboard</p>
            </div>
        </div>
    {/if}
</div>
