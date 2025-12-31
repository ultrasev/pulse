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

<div class="grid grid-cols-2 gap-4 flex-1 h-full">
    <!-- CPU Card -->
    <div class="p-5 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col items-center justify-center relative overflow-hidden group">
        <div class="absolute top-4 left-4 font-semibold text-gray-500 dark:text-gray-400 text-sm tracking-wider uppercase">CPU</div>

        {#if stats}
            {@const percent = stats.cpu_usage}
            {@const offset = circumference - (percent / 100) * circumference}

            <div class="relative w-32 h-32 flex items-center justify-center mt-4">
                <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                    <!-- Track -->
                    <circle
                        class="stroke-current transition-colors duration-500 {getTrackColor(percent)}"
                        stroke-width="8"
                        fill="transparent"
                        r={radius}
                        cx="50"
                        cy="50"
                    />
                    <!-- Progress -->
                    <circle
                        class="stroke-current transition-all duration-700 ease-out {getStatusColor(percent)}"
                        stroke-width="8"
                        stroke-linecap="round"
                        fill="transparent"
                        r={radius}
                        cx="50"
                        cy="50"
                        stroke-dasharray={circumference}
                        stroke-dashoffset={offset}
                    />
                </svg>
                <div class="absolute flex flex-col items-center">
                    <span class="text-3xl font-bold text-gray-800 dark:text-white tabular-nums tracking-tighter">
                        {Math.round(percent)}<span class="text-sm align-top text-gray-400 font-medium ml-0.5">%</span>
                    </span>
                </div>
            </div>
            <div class="mt-2 text-xs text-gray-400 font-mono">Usage</div>
        {:else}
            <div class="animate-pulse w-24 h-24 rounded-full bg-gray-200 dark:bg-gray-700 mt-4"></div>
        {/if}
    </div>

    <!-- Memory Card -->
    <div class="p-5 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col relative overflow-hidden">
        <div class="font-semibold text-gray-500 dark:text-gray-400 text-sm tracking-wider uppercase mb-1">Memory</div>

        {#if stats}
            {@const memPercent = (stats.memory_used / stats.memory_total) * 100}

            <div class="flex-1 flex flex-col justify-center gap-1">
                <div class="flex items-end justify-between mb-2">
                    <span class="text-2xl font-bold text-gray-800 dark:text-white">
                        {formatBytes(stats.memory_used)}
                    </span>
                    <span class="text-xs text-gray-400 mb-1">of {formatBytes(stats.memory_total)}</span>
                </div>

                <!-- Progress Bar -->
                <div class="h-3 w-full bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden">
                    <div
                        class="h-full rounded-full transition-all duration-700 ease-out {getStatusColor(memPercent).replace('stroke-', 'bg-')}"
                        style="width: {memPercent}%"
                    ></div>
                </div>

                <div class="flex justify-between text-xs text-gray-400 mt-2 font-mono">
                    <span>{memPercent.toFixed(1)}% Used</span>
                    <span>Free: {formatBytes(stats.memory_total - stats.memory_used)}</span>
                </div>
            </div>
        {:else}
             <div class="animate-pulse flex-1 flex flex-col justify-center gap-2">
                <div class="h-8 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded-full w-full"></div>
             </div>
        {/if}
    </div>

    <!-- Disk Card -->
    <div class="p-5 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col justify-between">
        <div class="font-semibold text-gray-500 dark:text-gray-400 text-sm tracking-wider uppercase">Disk</div>

        {#if stats}
            <div class="flex items-center gap-4 mt-2">
                <!-- Mini Pie Chart -->
                <div class="relative w-16 h-16 flex-shrink-0">
                     <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="text-gray-100 dark:text-gray-700 stroke-current" stroke-width="20" fill="transparent" r="40" cx="50" cy="50" />
                        <circle
                            class="stroke-current transition-all duration-700 ease-out text-blue-500"
                            stroke-width="20"
                            fill="transparent"
                            r="40"
                            cx="50"
                            cy="50"
                            stroke-dasharray={2 * Math.PI * 40}
                            stroke-dashoffset={2 * Math.PI * 40 - (stats.disk_usage_percent / 100) * 2 * Math.PI * 40}
                        />
                    </svg>
                </div>

                <div class="flex-1">
                    <div class="text-2xl font-bold text-gray-800 dark:text-white">
                        {stats.disk_usage_percent}%
                    </div>
                    <div class="text-xs text-gray-400">Storage Used</div>
                </div>
            </div>
        {:else}
            <div class="animate-pulse h-16 bg-gray-200 dark:bg-gray-700 rounded mt-2"></div>
        {/if}
    </div>

    <!-- Network Card -->
    <div class="p-5 bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col">
        <div class="font-semibold text-gray-500 dark:text-gray-400 text-sm tracking-wider uppercase mb-3">Network</div>

        {#if stats}
            <div class="flex-1 flex flex-col justify-center gap-4">
                <!-- Upload -->
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <div class="p-1.5 rounded-lg bg-orange-50 dark:bg-orange-900/20 text-orange-500">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path></svg>
                        </div>
                        <span class="text-xs font-medium text-gray-500 dark:text-gray-400">UP</span>
                    </div>
                    <span class="text-lg font-bold font-mono text-gray-800 dark:text-gray-200">{formatSpeed(stats.network_speed_up)}</span>
                </div>

                <!-- Download -->
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <div class="p-1.5 rounded-lg bg-green-50 dark:bg-green-900/20 text-green-500">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path></svg>
                        </div>
                        <span class="text-xs font-medium text-gray-500 dark:text-gray-400">DOWN</span>
                    </div>
                    <span class="text-lg font-bold font-mono text-gray-800 dark:text-gray-200">{formatSpeed(stats.network_speed_down)}</span>
                </div>
            </div>
        {:else}
            <div class="animate-pulse space-y-4">
                <div class="h-8 bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-8 bg-gray-200 dark:bg-gray-700 rounded"></div>
            </div>
        {/if}
    </div>
</div>
