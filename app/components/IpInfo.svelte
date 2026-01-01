<script lang="ts">
    import { onMount } from "svelte";
    import * as ipinfoService from "../lib/services/ipinfo";

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
            error = "Failed to load IP information";
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
            .split("")
            .map((char) => 127397 + char.charCodeAt(0));
        return String.fromCodePoint(...codePoints);
    };

    const copyToClipboard = async () => {
        if (ipData?.ip) {
            try {
                await navigator.clipboard.writeText(ipData.ip);
            } catch (e) {
                console.error("Failed to copy:", e);
            }
        }
    };

    onMount(() => {
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
    <div class="flex flex-col gap-5 flex-1">
        <!-- 主卡片 -->
        <div class="relative overflow-hidden rounded-2xl shadow-lg bg-cyan-600">
            <div class="relative p-6">
                <div class="flex items-center gap-3 mb-3">
                    <div class="flex items-center gap-2 px-3 py-1 bg-white/20 rounded-full">
                        <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                        <span class="text-xs text-white/90">Online</span>
                    </div>
                    {#if lastUpdate && !loading}
                        <span class="text-xs text-white/70">
                            {ipinfoService.getTimeAgo(lastUpdate)}
                        </span>
                    {/if}
                </div>

                <div class="text-xs text-white/70 mb-2 tracking-wide">PUBLIC IP ADDRESS</div>

                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-4">
                        {#if ipData?.country}
                            <span class="text-5xl drop-shadow-lg">{countryToFlag(ipData.country)}</span>
                        {/if}
                        <div class="text-4xl font-mono font-bold text-white tracking-wider drop-shadow">
                            {ipData?.ip}
                        </div>
                    </div>
                    <button
                        on:click={copyToClipboard}
                        class="p-3 bg-white/20 hover:bg-white/30 rounded-xl transition-all hover:scale-105"
                        title="复制 IP"
                    >
                        <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                        </svg>
                    </button>
                </div>
            </div>
        </div>

        <!-- 信息卡片网格 -->
        <div class="grid grid-cols-2 gap-4 flex-1">
            <!-- 位置 -->
            <div class="group p-5 bg-white rounded-2xl shadow-sm border border-gray-200 hover:shadow-md hover:border-cyan-300 transition-all">
                <div class="flex items-center gap-2 mb-3">
                    <div class="p-2 bg-cyan-100 rounded-lg">
                        <svg class="w-5 h-5 text-cyan-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"></path>
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"></path>
                        </svg>
                    </div>
                    <span class="text-xs font-medium text-gray-500 uppercase tracking-wide">Location</span>
                </div>
                <div class="space-y-1">
                    <div class="text-xl font-bold text-gray-900">
                        {#if ipData?.country}
                            <span class="mr-2">{countryToFlag(ipData.country)}</span>
                        {/if}
                        {ipData?.city || 'Unknown'}
                    </div>
                    <div class="text-sm text-gray-500">{ipData?.country}</div>
                    <div class="text-xs text-gray-400">{ipData?.region}</div>
                </div>
            </div>

            <!-- ISP -->
            <div class="group p-5 bg-white rounded-2xl shadow-sm border border-gray-200 hover:shadow-md hover:border-purple-300 transition-all">
                <div class="flex items-center gap-2 mb-3">
                    <div class="p-2 bg-purple-100 rounded-lg">
                        <svg class="w-5 h-5 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"></path>
                        </svg>
                    </div>
                    <span class="text-xs font-medium text-gray-500 uppercase tracking-wide">ISP</span>
                </div>
                <div class="space-y-1">
                    <div class="text-sm font-semibold text-gray-900 line-clamp-2">
                        {ipData?.ISP}
                    </div>
                    <div class="text-xs text-gray-400">ASN: {ipData?.ASN}</div>
                </div>
            </div>

            <!-- 坐标 -->
            <div class="group p-5 bg-white rounded-2xl shadow-sm border border-gray-200 hover:shadow-md hover:border-green-300 transition-all">
                <div class="flex items-center gap-2 mb-3">
                    <div class="p-2 bg-green-100 rounded-lg">
                        <svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"></path>
                        </svg>
                    </div>
                    <span class="text-xs font-medium text-gray-500 uppercase tracking-wide">Coordinates</span>
                </div>
                <div class="font-mono text-sm space-y-1">
                    <div class="flex items-center gap-2">
                        <span class="text-gray-400">N</span>
                        <span class="text-gray-900">{ipData?.latitude || '0.00'}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-gray-400">E</span>
                        <span class="text-gray-900">{ipData?.longitude || '0.00'}</span>
                    </div>
                </div>
            </div>

            <!-- 时区 -->
            <div class="group p-5 bg-white rounded-2xl shadow-sm border border-gray-200 hover:shadow-md hover:border-orange-300 transition-all">
                <div class="flex items-center gap-2 mb-3">
                    <div class="p-2 bg-orange-100 rounded-lg">
                        <svg class="w-5 h-5 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                        </svg>
                    </div>
                    <span class="text-xs font-medium text-gray-500 uppercase tracking-wide">Timezone</span>
                </div>
                <div class="space-y-1">
                    <div class="text-lg font-bold text-gray-900">
                        {ipData?.timezone}
                    </div>
                    <div class="text-xs text-gray-400">Postal: {ipData?.postalCode || 'N/A'}</div>
                </div>
            </div>
        </div>

        <!-- 刷新按钮 -->
        <button
            on:click={() => fetchIpInfo()}
            disabled={isRefreshing}
            class="group relative w-full py-3.5 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 rounded-2xl transition-all font-medium flex items-center justify-center gap-2 overflow-hidden"
        >
            {#if isRefreshing}
                <div class="animate-spin rounded-full h-5 w-5 border-2 border-gray-500 border-t-transparent"></div>
                <span>刷新中...</span>
            {:else}
                <svg class="w-5 h-5 text-gray-600 group-hover:rotate-180 transition-transform duration-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                </svg>
                <span>Refresh</span>
            {/if}
        </button>
    </div>
{/if}
