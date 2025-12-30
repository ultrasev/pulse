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
</script>

<div class="grid grid-cols-2 gap-4 flex-1">
    <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold mb-2 text-blue-500">CPU</h2>
        <div class="text-3xl font-bold">{stats ? `${stats.cpu_usage.toFixed(1)}%` : '--%'}</div>
    </div>

    <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold mb-2 text-purple-500">Memory</h2>
        <div class="text-3xl font-bold">
            {stats ? `${formatBytes(stats.memory_used)} / ${formatBytes(stats.memory_total)}` : '-- / --'}
        </div>
    </div>

    <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold mb-2 text-green-500">Disk</h2>
        <div class="text-3xl font-bold">{stats ? `${stats.disk_usage_percent}%` : '--%'}</div>
    </div>

    <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold mb-2 text-orange-500">Network</h2>
        <div class="flex flex-col text-sm">
            <span>↑ {stats ? formatSpeed(stats.network_speed_up) : '--'}</span>
            <span>↓ {stats ? formatSpeed(stats.network_speed_down) : '--'}</span>
        </div>
    </div>
</div>
