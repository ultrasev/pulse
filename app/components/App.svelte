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

    const tabs: { id: Tab; label: string; icon: string; activeClass: string }[] = [
        { id: "system", label: "System", icon: "📊", activeClass: "bg-blue-100 text-blue-600" },
        { id: "ipinfo", label: "IP Info", icon: "🌐", activeClass: "bg-cyan-100 text-cyan-600" },
        { id: "upload", label: "Upload", icon: "📤", activeClass: "bg-purple-100 text-purple-600" },
        { id: "git", label: "Claude Models", icon: "🤖", activeClass: "bg-red-100 text-red-600" },
    ];

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

<div class="h-full flex">
    <aside class="w-52 shrink-0 flex flex-col p-4 gap-4 bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800">
        <h1 class="text-2xl font-bold px-2">Pulse</h1>

        <nav class="flex flex-col gap-1">
            {#each tabs as tab}
                <button
                    on:click={() => (activeTab = tab.id)}
                    class="flex items-center gap-3 py-2 px-3 rounded-md font-medium text-left transition-colors
                        {activeTab === tab.id
                        ? tab.activeClass
                        : 'text-gray-500 hover:text-gray-700 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:bg-gray-800'}"
                >
                    <span>{tab.icon}</span>
                    <span>{tab.label}</span>
                </button>
            {/each}
        </nav>

        <div class="mt-auto flex items-center justify-between px-2">
            <div class="text-xs text-gray-500">v0.0.6</div>
            <button
                on:click={toggleTheme}
                class="p-2 rounded-lg bg-gray-200 hover:bg-gray-300 transition-colors"
                title={isDark ? "Switch to Light" : "Switch to Dark"}
            >
                {#if isDark}☀️{:else}🌙{/if}
            </button>
        </div>
    </aside>

    <main class="flex-1 p-4 overflow-y-auto">
        <div class:hidden={activeTab !== "system"}>
            <SystemDashboard />
        </div>
        <div class:hidden={activeTab !== "ipinfo"}>
            <IpInfo />
        </div>
        <div class:hidden={activeTab !== "upload"}>
            <ImageUpload />
        </div>
        <div class:hidden={activeTab !== "git"}>
            <GitManager />
        </div>
    </main>
</div>
