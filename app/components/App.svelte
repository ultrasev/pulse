<script lang="ts">
    import SystemDashboard from './SystemDashboard.svelte';
    import IpInfo from './IpInfo.svelte';
    import ProductPrice from './ProductPrice.svelte';
    import ImageUpload from './ImageUpload.svelte';
    import { listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte';

    type Tab = 'system' | 'ipinfo' | 'price' | 'upload';

    let activeTab: Tab = 'system';

    onMount(() => {
        // Listen for switch-to-upload event from Rust
        const unlisten = listen('switch-to-upload', () => {
            activeTab = 'upload';
        });
        return () => {
            unlisten.then(fn => fn());
        };
    });
</script>

<div class="h-full flex flex-col p-4 gap-4">
    <header class="flex justify-between items-center">
        <h1 class="text-2xl font-bold">System Monitor</h1>
        <div class="text-xs text-gray-500">v0.0.1</div>
    </header>

    <nav class="flex gap-2 p-1 bg-gray-100 dark:bg-gray-800 rounded-lg">
        <button
            on:click={() => activeTab = 'system'}
            class:flex-1={true}
            class="py-2 px-4 rounded-md font-medium transition-all"
            class:bg-white={activeTab === 'system'}
            class:dark:bg-gray-700={activeTab === 'system'}
            class:shadow-sm={activeTab === 'system'}
            class:text-blue-600={activeTab === 'system'}
            class:dark:text-blue-400={activeTab === 'system'}
            class:text-gray-500={activeTab !== 'system'}
            class:hover:text-gray-700={activeTab !== 'system'}
            class:dark:hover:text-gray-300={activeTab !== 'system'}
        >
            📊 System
        </button>
        <button
            on:click={() => activeTab = 'ipinfo'}
            class:flex-1={true}
            class="py-2 px-4 rounded-md font-medium transition-all"
            class:bg-white={activeTab === 'ipinfo'}
            class:dark:bg-gray-700={activeTab === 'ipinfo'}
            class:shadow-sm={activeTab === 'ipinfo'}
            class:text-cyan-600={activeTab === 'ipinfo'}
            class:dark:text-cyan-400={activeTab === 'ipinfo'}
            class:text-gray-500={activeTab !== 'ipinfo'}
            class:hover:text-gray-700={activeTab !== 'ipinfo'}
            class:dark:hover:text-gray-300={activeTab !== 'ipinfo'}
        >
            🌐 IP Info
        </button>
        <button
            on:click={() => activeTab = 'price'}
            class:flex-1={true}
            class="py-2 px-4 rounded-md font-medium transition-all"
            class:bg-white={activeTab === 'price'}
            class:dark:bg-gray-700={activeTab === 'price'}
            class:shadow-sm={activeTab === 'price'}
            class:text-orange-600={activeTab === 'price'}
            class:dark:text-orange-400={activeTab === 'price'}
            class:text-gray-500={activeTab !== 'price'}
            class:hover:text-gray-700={activeTab !== 'price'}
            class:dark:hover:text-gray-300={activeTab !== 'price'}
        >
            🏷️ Price
        </button>
        <button
            on:click={() => activeTab = 'upload'}
            class:flex-1={true}
            class="py-2 px-4 rounded-md font-medium transition-all"
            class:bg-white={activeTab === 'upload'}
            class:dark:bg-gray-700={activeTab === 'upload'}
            class:shadow-sm={activeTab === 'upload'}
            class:text-purple-600={activeTab === 'upload'}
            class:dark:text-purple-400={activeTab === 'upload'}
            class:text-gray-500={activeTab !== 'upload'}
            class:hover:text-gray-700={activeTab !== 'upload'}
            class:dark:hover:text-gray-300={activeTab !== 'upload'}
        >
            📤 Upload
        </button>
    </nav>

    {#if activeTab === 'system'}
        <SystemDashboard />
    {:else if activeTab === 'ipinfo'}
        <IpInfo />
    {:else if activeTab === 'price'}
        <ProductPrice />
    {:else if activeTab === 'upload'}
        <ImageUpload />
    {/if}
</div>
