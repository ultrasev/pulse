<script lang="ts">
    import SystemDashboard from "./SystemDashboard.svelte";
    import IpInfo from "./IpInfo.svelte";
    import ProductPrice from "./ProductPrice.svelte";
    import ImageUpload from "./ImageUpload.svelte";
    import GitManager from "./GitManager.svelte";
    import { listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";

    type Tab = "system" | "ipinfo" | "upload" | "git";

    let activeTab: Tab = "system";
    let isDark = false;

    function toggleTheme() {
        isDark = document.documentElement.classList.toggle("dark");
        localStorage.setItem("theme", isDark ? "dark" : "light");
    }

    onMount(() => {
        isDark = document.documentElement.classList.contains("dark");

        const unlisten = listen("switch-to-upload", () => {
            activeTab = "upload";
        });
        return () => {
            unlisten.then((fn) => fn());
        };
    });
</script>

<div class="h-full flex flex-col p-4 gap-4">
    <header class="flex justify-between items-center">
        <h1 class="text-2xl font-bold">Pulse</h1>
        <div class="flex items-center gap-3">
            <button
                on:click={toggleTheme}
                class="p-2 rounded-lg bg-gray-200 hover:bg-gray-300 transition-colors"
                title={isDark ? "Switch to Light" : "Switch to Dark"}
            >
                {#if isDark}☀️{:else}🌙{/if}
            </button>
            <div class="text-xs text-gray-500">v0.0.4</div>
        </div>
    </header>

    <nav class="flex gap-2 p-1 bg-gray-100 rounded-lg">
        <button
            on:click={() => (activeTab = "system")}
            class="flex-1 py-2 px-4 rounded-md font-medium transition-all {activeTab === 'system' ? 'bg-white shadow-sm text-blue-600' : 'text-gray-500 hover:text-gray-700'}"
        >
            📊 System
        </button>
        <button
            on:click={() => (activeTab = "ipinfo")}
            class="flex-1 py-2 px-4 rounded-md font-medium transition-all {activeTab === 'ipinfo' ? 'bg-white shadow-sm text-cyan-600' : 'text-gray-500 hover:text-gray-700'}"
        >
            🌐 IP Info
        </button>
        <button
            on:click={() => (activeTab = "upload")}
            class="flex-1 py-2 px-4 rounded-md font-medium transition-all {activeTab === 'upload' ? 'bg-white shadow-sm text-purple-600' : 'text-gray-500 hover:text-gray-700'}"
        >
            📤 Upload
        </button>
        <button
            on:click={() => (activeTab = "git")}
            class="flex-1 py-2 px-4 rounded-md font-medium transition-all {activeTab === 'git' ? 'bg-white shadow-sm text-red-600' : 'text-gray-500 hover:text-gray-700'}"
        >
            🤖 Claude Models
        </button>
    </nav>

    {#if activeTab === "system"}
        <SystemDashboard />
    {:else if activeTab === "ipinfo"}
        <IpInfo />
    {:else if activeTab === "upload"}
        <ImageUpload />
    {:else if activeTab === "git"}
        <GitManager />
    {/if}
</div>
