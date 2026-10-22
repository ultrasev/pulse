<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import * as zai from '../lib/services/zai';

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

    let zaiQuota: zai.ZaiQuota | null = null;
    let zaiError = '';
    let zaiToken = '';
    let showZaiConfig = false;

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
        if (percent >= 80) return 'stroke-red-100';
        if (percent >= 50) return 'stroke-orange-100';
        return 'stroke-green-100';
    };

    const fetchStats = async () => {
        try {
            const data = await invoke<SystemStats>('get_system_stats');
            stats = data;
        } catch (e) {
            console.error("Failed to fetch stats:", e);
        }
    };

    const fetchZaiQuota = async () => {
        try {
            zaiError = '';
            zaiQuota = await zai.fetchZaiQuota();
        } catch (e) {
            zaiError = e instanceof Error ? e.message : String(e);
        }
    };

    const saveZaiToken = () => {
        zai.setBearer(zaiToken.trim());
        showZaiConfig = false;
        fetchZaiQuota();
    };

    const formatResetIn = (minutes: number | null): string => {
        if (minutes == null) return 'N/A';
        const h = Math.floor(minutes / 60);
        const m = minutes % 60;
        return `${h}h ${m}m`;
    };

    onMount(() => {
        fetchStats();
        intervalId = setInterval(fetchStats, 5000);
        zaiToken = zai.getBearer();
        if (zai.hasBearer()) fetchZaiQuota();
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
        <div class="p-4 bg-white rounded-2xl shadow-sm border border-gray-200 flex flex-col items-center justify-center relative overflow-hidden h-48">
            <div class="absolute top-3 left-4 font-semibold text-gray-500 text-xs tracking-wider uppercase">CPU</div>

            {#if stats}
                {@const percent = stats.cpu_usage}
                {@const offset = circumference - (percent / 100) * circumference}

                <div class="relative w-28 h-28 flex items-center justify-center mt-2">
                    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="stroke-current transition-colors duration-500 {getTrackColor(percent)}" stroke-width="8" fill="transparent" r={radius} cx="50" cy="50" />
                        <circle class="stroke-current transition-all duration-700 ease-out {getStatusColor(percent)}" stroke-width="8" stroke-linecap="round" fill="transparent" r={radius} cx="50" cy="50" stroke-dasharray={circumference} stroke-dashoffset={offset} />
                    </svg>
                    <div class="absolute flex flex-col items-center">
                        <span class="text-2xl font-bold text-gray-800 tabular-nums tracking-tighter">
                            {Math.round(percent)}<span class="text-xs align-top text-gray-400 font-medium ml-0.5">%</span>
                        </span>
                    </div>
                </div>
                <div class="mt-1 text-[10px] text-gray-400 font-mono">Usage</div>
            {:else}
                <div class="animate-pulse w-24 h-24 rounded-full bg-gray-200 mt-2"></div>
            {/if}
        </div>

        <!-- Memory Card -->
        <div class="p-4 bg-white rounded-2xl shadow-sm border border-gray-200 flex flex-col items-center justify-center relative overflow-hidden h-48">
            <div class="absolute top-3 left-4 font-semibold text-gray-500 text-xs tracking-wider uppercase">Memory</div>

            {#if stats}
                {@const percent = (stats.memory_used / stats.memory_total) * 100}
                {@const offset = circumference - (percent / 100) * circumference}

                <div class="relative w-28 h-28 flex items-center justify-center mt-2">
                    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="stroke-current transition-colors duration-500 {getTrackColor(percent)}" stroke-width="8" fill="transparent" r={radius} cx="50" cy="50" />
                        <circle class="stroke-current transition-all duration-700 ease-out {getStatusColor(percent)}" stroke-width="8" stroke-linecap="round" fill="transparent" r={radius} cx="50" cy="50" stroke-dasharray={circumference} stroke-dashoffset={offset} />
                    </svg>
                    <div class="absolute flex flex-col items-center">
                        <span class="text-2xl font-bold text-gray-800 tabular-nums tracking-tighter">
                            {Math.round(percent)}<span class="text-xs align-top text-gray-400 font-medium ml-0.5">%</span>
                        </span>
                    </div>
                </div>
                <div class="mt-1 text-[10px] text-gray-400 font-mono">{formatBytes(stats.memory_used)} / {formatBytes(stats.memory_total)}</div>
            {:else}
                <div class="animate-pulse w-24 h-24 rounded-full bg-gray-200 mt-2"></div>
            {/if}
        </div>

        <!-- Disk Card -->
        <div class="p-4 bg-white rounded-2xl shadow-sm border border-gray-200 flex flex-col items-center justify-center relative overflow-hidden h-48">
            <div class="absolute top-3 left-4 font-semibold text-gray-500 text-xs tracking-wider uppercase">Disk</div>

            {#if stats}
                {@const percent = stats.disk_usage_percent}
                {@const offset = circumference - (percent / 100) * circumference}

                <div class="relative w-28 h-28 flex items-center justify-center mt-2">
                    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="stroke-current transition-colors duration-500 {getTrackColor(percent)}" stroke-width="8" fill="transparent" r={radius} cx="50" cy="50" />
                        <circle class="stroke-current transition-all duration-700 ease-out {getStatusColor(percent)}" stroke-width="8" stroke-linecap="round" fill="transparent" r={radius} cx="50" cy="50" stroke-dasharray={circumference} stroke-dashoffset={offset} />
                    </svg>
                    <div class="absolute flex flex-col items-center">
                        <span class="text-2xl font-bold text-gray-800 tabular-nums tracking-tighter">
                            {percent}<span class="text-xs align-top text-gray-400 font-medium ml-0.5">%</span>
                        </span>
                    </div>
                </div>
                <div class="mt-1 text-[10px] text-gray-400 font-mono">Used Space</div>
            {:else}
                <div class="animate-pulse w-24 h-24 rounded-full bg-gray-200 mt-2"></div>
            {/if}
        </div>
    </div>

    <!-- Bottom Row: Network -->
    <div class="p-5 bg-white rounded-2xl shadow-sm border border-gray-200 flex flex-col justify-center">
        <div class="font-semibold text-gray-500 text-sm tracking-wider uppercase mb-2">Network</div>

        {#if stats}
            <div class="font-mono text-lg text-gray-800 space-y-1">
                <div>Upload: {formatSpeed(stats.network_speed_up)}</div>
                <div>Download: {formatSpeed(stats.network_speed_down)}</div>
            </div>
        {:else}
            <div class="animate-pulse space-y-2">
                <div class="h-6 bg-gray-200 rounded w-1/3"></div>
                <div class="h-6 bg-gray-200 rounded w-1/3"></div>
            </div>
        {/if}
    </div>

    <!-- ZAI Subscription -->
    <div class="p-5 bg-white rounded-2xl shadow-sm border border-gray-200 flex flex-col justify-center">
        <div class="flex justify-between items-center mb-2">
            <div class="font-semibold text-gray-500 text-sm tracking-wider uppercase">Z.AI Subscription</div>
            <button
                class="text-xs text-gray-400 hover:text-gray-600"
                on:click={() => (showZaiConfig = !showZaiConfig)}
                title="配置 Bearer Token"
            >⚙️</button>
        </div>

        {#if showZaiConfig}
            <div class="flex gap-2 mb-3">
                <input
                    type="password"
                    bind:value={zaiToken}
                    placeholder="Bearer Token"
                    class="flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded-lg focus:outline-none focus:border-blue-400"
                />
                <button
                    class="px-3 py-1.5 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                    on:click={saveZaiToken}
                >保存</button>
            </div>
        {/if}

        {#if zaiQuota}
            {@const zaiPercent = Math.min(100, zaiQuota.percentage)}
            {@const zaiOffset = circumference - (zaiPercent / 100) * circumference}
            <div class="flex items-center gap-6">
                <div class="relative w-24 h-24 flex items-center justify-center shrink-0">
                    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
                        <circle class="stroke-gray-100" stroke-width="8" fill="transparent" r={radius} cx="50" cy="50" />
                        <circle
                            class="transition-all duration-700 ease-out
                                {zaiPercent >= 90 ? 'stroke-red-500' : zaiPercent >= 80 ? 'stroke-orange-500' : 'stroke-green-500'}"
                            stroke-width="8" stroke-linecap="round" fill="transparent"
                            r={radius} cx="50" cy="50"
                            stroke-dasharray={circumference} stroke-dashoffset={zaiOffset}
                        />
                    </svg>
                    <div class="absolute flex flex-col items-center">
                        <span class="text-xl font-bold text-gray-800 tabular-nums tracking-tighter">
                            {zaiPercent}<span class="text-xs align-top text-gray-400 font-medium ml-0.5">%</span>
                        </span>
                    </div>
                </div>
                <div class="text-gray-800 space-y-1 font-sans">
                    <div class="text-base">reset in <span class="font-semibold">{formatResetIn(zaiQuota.resetInMinutes)}</span></div>
                    <div class="text-sm text-gray-400">{zaiQuota.resetAt ?? 'N/A'}</div>
                </div>
            </div>
        {:else if zaiError}
            <div class="text-sm text-red-500 font-mono">{zaiError}</div>
        {:else if !zaiToken}
            <div class="text-sm text-gray-400">点击 ⚙️ 配置 Bearer Token</div>
        {:else}
            <div class="animate-pulse h-6 bg-gray-200 rounded w-1/3"></div>
        {/if}
    </div>
</div>
