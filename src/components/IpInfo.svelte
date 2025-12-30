<script lang="ts">
    import { onMount } from 'svelte';
    import * as ipinfoService from '../lib/services/ipinfo';

    let ipData: ipinfoService.IpData | null = null;
    let loading = false;
    let isRefreshing = false;
    let error: string | null = null;
    let lastUpdate: number | null = null;

    const fetchIpInfo = async (isBackground = false) => {
        if (!isBackground) {
            loading = true;
        } else {
            isRefreshing = true;
        }
        error = null;
        try {
            const data = await ipinfoService.fetchIpInfo();
            ipData = data;
            lastUpdate = Date.now();
        } catch (e) {
            console.error("Failed to fetch IP info:", e);
            error = 'Failed to load IP information';
        } finally {
            if (!isBackground) {
                loading = false;
            } else {
                isRefreshing = false;
            }
        }
    };

    const countryToFlag = (countryCode: string) => {
        const codePoints = countryCode
            .toUpperCase()
            .split('')
            .map(char => 127397 + char.charCodeAt(0));
        return String.fromCodePoint(...codePoints);
    };

    onMount(() => {
        // Try to load from cache first
        const cached = ipinfoService.getCached();
        if (cached) {
            ipData = cached;
            lastUpdate = ipinfoService.getCacheTimestamp();
            fetchIpInfo(true);
        } else {
            fetchIpInfo();
        }
    });
</script>

{#if loading && !ipData}
    <div class="flex items-center justify-center h-64">
        <div class="animate-spin rounded-full h-12 w-12 border-4 border-cyan-500 border-t-transparent"></div>
    </div>
{:else if error && !ipData}
    <div class="flex flex-col items-center justify-center h-64 gap-4">
        <div class="text-red-500 text-lg">{error}</div>
        <button
            on:click={() => fetchIpInfo()}
            class="px-4 py-2 bg-cyan-500 text-white rounded-lg hover:bg-cyan-600 transition-colors"
        >
            Retry
        </button>
    </div>
{:else}
    <div class="flex flex-col gap-4 flex-1">
        <div class="p-6 bg-gradient-to-br from-cyan-500 to-blue-600 rounded-2xl shadow-lg text-white">
            <div class="text-sm opacity-80 mb-1">Public IP Address</div>
            <div class="text-3xl font-mono font-bold tracking-wider">{ipData?.ip}</div>
        </div>

        {#if lastUpdate && !loading}
            <div class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 px-2">
                <span>更新于 {ipinfoService.getTimeAgo(lastUpdate)}</span>
                {#if isRefreshing}
                    <span class="text-cyan-500">更新中...</span>
                {/if}
            </div>
        {/if}

        <div class="grid grid-cols-2 gap-3 flex-1">
            <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">Location</div>
                <div class="text-lg font-semibold flex items-center gap-2">
                    {#if ipData?.country}
                        <span class="text-2xl">{countryToFlag(ipData.country)}</span>
                    {/if}
                    <span>{ipData?.city}, {ipData?.country}</span>
                </div>
                <div class="text-sm text-gray-500">{ipData?.region}</div>
            </div>

            <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">ISP</div>
                <div class="text-sm font-semibold line-clamp-2">{ipData?.ISP}</div>
                <div class="text-sm text-gray-500 mt-1">ASN: {ipData?.ASN}</div>
            </div>

            <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">Coordinates</div>
                <div class="font-mono text-sm">
                    <div>Lat: {ipData?.latitude}</div>
                    <div>Lon: {ipData?.longitude}</div>
                </div>
            </div>

            <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">Timezone</div>
                <div class="text-lg font-semibold">{ipData?.timezone}</div>
                <div class="text-sm text-gray-500">Postal: {ipData?.postalCode || 'N/A'}</div>
            </div>
        </div>

        <button
            on:click={() => fetchIpInfo()}
            class="w-full py-3 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-xl transition-colors font-medium"
        >
            ↻ Refresh
        </button>
    </div>
{/if}
