<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    interface SystemStats {
        cpu_usage: number;
        memory_used: number;
        memory_total: number;
        disk_usage_percent: number;
        network_speed_up: number;
        network_speed_down: number;
    }

    let stats: SystemStats | null = null;
    let intervalId: ReturnType<typeof setInterval> | null = null;

    const formatBytes = (bytes: number) => {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };

    const formatSpeed = (bytes: number) => formatBytes(bytes) + '/s';

    // Helper to determine color based on percentage
    const getStatusColor = (percent: number) => {
        if (percent >= 80) return 'text-red-500 stroke-red-500';
        if (percent >= 50) return 'text-orange-500 stroke-orange-500';
        return 'text-green-500 stroke-green-500';
    };

    // For background track colors
    const getTrackColor = (percent: number) => {
        if (percent >= 80) return 'stroke-red-100 dark:stroke-red-900/30';
        if (percent >= 50) return 'stroke-orange-100 dark:stroke-orange-900/30';
        return 'stroke-green-100 dark:stroke-green-900/30';
    };

    const fetchStats = async () => {
        try {
            const data = await invoke<SystemStats>('get_system_stats');
            stats = data;
        } catch (e) {
            console.error("Failed to fetch stats:", e);
        }
    };

    onMount(() => {
        fetchStats();
        intervalId = setInterval(fetchStats, 1000);
    });

    onDestroy(() => {
        if (intervalId) clearInterval(intervalId);
    });

    // Circular progress helpers
    const radius = 36;
    const circumference = 2 * Math.PI * radius;
</script>

<div class="flex flex-col gap-4 flex-1 h-full">
    <!-- Top Row: Gauges (CPU, Memory, Disk) -->
    <div class="grid grid-cols-3 gap-4">
        <!-- CPU Card -->
        <div class="p-4 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col items-center justify-center relative overflow-hidden h-48">
            <div class="absolute top-3 left-4 font-semibold text-gray-500 dark:text-gray-400 text-xs tracking-wider uppercase">CPU</div>

            {#if stats}
                {@const percent = stats.cpu_usage}
                {@const offset = circumference - (percent / 100) * circumference}

                <div class="relative w-28 h-28 flex items-center justify-center mt-2">
                    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="stroke-current transition-colors duration-500 {getTrackColor(percent)}" stroke-width="8" fill="transparent" r={radius} cx="50" cy="50" />
                        <circle class="stroke-current transition-all duration-700 ease-out {getStatusColor(percent)}" stroke-width="8" stroke-linecap="round" fill="transparent" r={radius} cx="50" cy="50" stroke-dasharray={circumference} stroke-dashoffset={offset} />
                    </svg>
                    <div class="absolute flex flex-col items-center">
                        <span class="text-2xl font-bold text-gray-800 dark:text-white tabular-nums tracking-tighter">
                            {Math.round(percent)}<span class="text-xs align-top text-gray-400 font-medium ml-0.5">%</span>
                        </span>
                    </div>
                </div>
                <div class="mt-1 text-[10px] text-gray-400 font-mono">Usage</div>
            {:else}
                <div class="animate-pulse w-24 h-24 rounded-full bg-gray-200 dark:bg-gray-700 mt-2"></div>
            {/if}
        </div>

        <!-- Memory Card -->
        <div class="p-4 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col items-center justify-center relative overflow-hidden h-48">
            <div class="absolute top-3 left-4 font-semibold text-gray-500 dark:text-gray-400 text-xs tracking-wider uppercase">Memory</div>

            {#if stats}
                {@const percent = (stats.memory_used / stats.memory_total) * 100}
                {@const offset = circumference - (percent / 100) * circumference}

                <div class="relative w-28 h-28 flex items-center justify-center mt-2">
                    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="stroke-current transition-colors duration-500 {getTrackColor(percent)}" stroke-width="8" fill="transparent" r={radius} cx="50" cy="50" />
                        <circle class="stroke-current transition-all duration-700 ease-out {getStatusColor(percent)}" stroke-width="8" stroke-linecap="round" fill="transparent" r={radius} cx="50" cy="50" stroke-dasharray={circumference} stroke-dashoffset={offset} />
                    </svg>
                    <div class="absolute flex flex-col items-center">
                        <span class="text-2xl font-bold text-gray-800 dark:text-white tabular-nums tracking-tighter">
                            {Math.round(percent)}<span class="text-xs align-top text-gray-400 font-medium ml-0.5">%</span>
                        </span>
                    </div>
                </div>
                <div class="mt-1 text-[10px] text-gray-400 font-mono">{formatBytes(stats.memory_used)} / {formatBytes(stats.memory_total)}</div>
            {:else}
                <div class="animate-pulse w-24 h-24 rounded-full bg-gray-200 dark:bg-gray-700 mt-2"></div>
            {/if}
        </div>

        <!-- Disk Card -->
        <div class="p-4 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col items-center justify-center relative overflow-hidden h-48">
            <div class="absolute top-3 left-4 font-semibold text-gray-500 dark:text-gray-400 text-xs tracking-wider uppercase">Disk</div>

            {#if stats}
                {@const percent = stats.disk_usage_percent}
                {@const offset = circumference - (percent / 100) * circumference}

                <div class="relative w-28 h-28 flex items-center justify-center mt-2">
                    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="stroke-current transition-colors duration-500 {getTrackColor(percent)}" stroke-width="8" fill="transparent" r={radius} cx="50" cy="50" />
                        <circle class="stroke-current transition-all duration-700 ease-out {getStatusColor(percent)}" stroke-width="8" stroke-linecap="round" fill="transparent" r={radius} cx="50" cy="50" stroke-dasharray={circumference} stroke-dashoffset={offset} />
                    </svg>
                    <div class="absolute flex flex-col items-center">
                        <span class="text-2xl font-bold text-gray-800 dark:text-white tabular-nums tracking-tighter">
                            {percent}<span class="text-xs align-top text-gray-400 font-medium ml-0.5">%</span>
                        </span>
                    </div>
                </div>
                <div class="mt-1 text-[10px] text-gray-400 font-mono">Used Space</div>
            {:else}
                <div class="animate-pulse w-24 h-24 rounded-full bg-gray-200 dark:bg-gray-700 mt-2"></div>
            {/if}
        </div>
    </div>

    <!-- Bottom Row: Network -->
    <div class="p-5 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col flex-1 min-h-0 justify-center">
        <div class="font-semibold text-gray-500 dark:text-gray-400 text-sm tracking-wider uppercase mb-4">Network Activity</div>

        {#if stats}
            <div class="grid grid-cols-2 gap-8 h-full items-center">
                <!-- Upload -->
                <div class="flex flex-col items-center justify-center p-4 rounded-xl bg-orange-50/50 dark:bg-orange-900/10 border border-orange-100 dark:border-orange-800/30">
                    <div class="flex items-center gap-2 mb-2">
                        <div class="p-1.5 rounded-lg bg-orange-100 dark:bg-orange-900/30 text-orange-500">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path></svg>
                        </div>
                        <span class="text-sm font-medium text-orange-600 dark:text-orange-400">UPLOAD</span>
                    </div>
                    <span class="text-3xl font-bold font-mono text-gray-800 dark:text-gray-200 tracking-tight">{formatSpeed(stats.network_speed_up)}</span>
                </div>

                <!-- Download -->
                <div class="flex flex-col items-center justify-center p-4 rounded-xl bg-green-50/50 dark:bg-green-900/10 border border-green-100 dark:border-green-800/30">
                    <div class="flex items-center gap-2 mb-2">
                        <div class="p-1.5 rounded-lg bg-green-100 dark:bg-green-900/30 text-green-500">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path></svg>
                        </div>
                        <span class="text-sm font-medium text-green-600 dark:text-green-400">DOWNLOAD</span>
                    </div>
                    <span class="text-3xl font-bold font-mono text-gray-800 dark:text-gray-200 tracking-tight">{formatSpeed(stats.network_speed_down)}</span>
                </div>
            </div>
        {:else}
            <div class="animate-pulse grid grid-cols-2 gap-8 h-full">
                <div class="bg-gray-200 dark:bg-gray-700 rounded-xl h-full"></div>
                <div class="bg-gray-200 dark:bg-gray-700 rounded-xl h-full"></div>
            </div>
        {/if}
    </div>
</div>
