<script lang="ts">
    import SystemDashboard from './SystemDashboard.svelte';
    import IpInfo from './IpInfo.svelte';
    import ProductPrice from './ProductPrice.svelte';

    type Tab = 'system' | 'ipinfo' | 'price';

    let activeTab: Tab = 'system';
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
    </nav>

    {#if activeTab === 'system'}
        <SystemDashboard />
    {:else if activeTab === 'ipinfo'}
        <IpInfo />
    {:else if activeTab === 'price'}
        <ProductPrice />
    {/if}
</div>
