<script lang="ts">
    import { onMount } from 'svelte';
    import * as productService from '../lib/services/product';

    let skuId = '100209267857';
    let inputSku = '100209267857';
    let productData: productService.ProductData | null = null;
    let loading = false;
    let isRefreshing = false;
    let error: string | null = null;
    let lastUpdate: number | null = null;

    const fetchProductPrice = async (sku: string, isBackground = false) => {
        if (!isBackground) {
            loading = true;
        } else {
            isRefreshing = true;
        }
        error = null;
        try {
            const data = await productService.fetchProductPrice(sku);
            productData = data;
            skuId = sku;
            lastUpdate = Date.now();
        } catch (e) {
            console.error("Failed to fetch product info:", e);
            error = e instanceof Error ? e.message : '加载商品信息失败';
        } finally {
            if (!isBackground) {
                loading = false;
            } else {
                isRefreshing = false;
            }
        }
    };

    const handleSubmit = () => {
        if (inputSku.trim()) {
            const sku = inputSku.trim();
            const cached = productService.getCached(sku);
            if (cached) {
                productData = cached;
                skuId = sku;
                lastUpdate = productService.getCacheTimestamp(sku);
            }
            fetchProductPrice(sku, !!cached);
        }
    };

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit'
        });
    };

    onMount(() => {
        const cached = productService.getCached(skuId);
        if (cached) {
            productData = cached;
            lastUpdate = productService.getCacheTimestamp(skuId);
            fetchProductPrice(skuId, true);
        } else {
            fetchProductPrice(skuId);
        }
    });
</script>

<div class="flex flex-col gap-4 flex-1">
    <form on:submit|preventDefault={handleSubmit} class="flex gap-2">
        <input
            type="text"
            bind:value={inputSku}
            placeholder="输入京东 SKU ID"
            class="flex-1 px-4 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500"
        />
        <button
            type="submit"
            disabled={loading}
            class="px-6 py-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
        >
            {loading ? '查询中...' : '查询'}
        </button>
    </form>

    {#if lastUpdate && !loading}
        <div class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 px-2">
            <span>更新于 {productService.getTimeAgo(lastUpdate)}</span>
            {#if isRefreshing}
                <span class="text-orange-500">更新中...</span>
            {/if}
        </div>
    {/if}

    {#if loading && !productData}
        <div class="flex items-center justify-center h-64">
            <div class="animate-spin rounded-full h-12 w-12 border-4 border-orange-500 border-t-transparent"></div>
        </div>
    {:else if error && !productData}
        <div class="flex flex-col items-center justify-center h-64 gap-4">
            <div class="text-red-500 text-lg">{error}</div>
            <button
                on:click={() => fetchProductPrice(inputSku)}
                class="px-4 py-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors"
            >
                重试
            </button>
        </div>
    {:else if productData && !loading}
        <div class="flex flex-col gap-4 flex-1 overflow-auto">
            <div class="flex gap-4 p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                {#if productData.data.product.imageUrl}
                    <img
                        src={productData.data.product.imageUrl}
                        alt={productData.data.product.name}
                        class="w-24 h-24 object-contain rounded-lg"
                    />
                {/if}
                <div class="flex-1">
                    <h2 class="text-xl font-bold mb-1">{productData.data.product.name}</h2>
                    <div class="text-sm text-gray-500 dark:text-gray-400 mb-2">
                        {productData.data.product.variant}
                    </div>
                    <a
                        href={productData.data.product.jdUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-sm text-orange-500 hover:underline"
                    >
                        查看京东商品页 →
                    </a>
                </div>
            </div>

            <div class="p-6 bg-orange-500 rounded-2xl shadow-lg text-white">
                <div class="text-sm opacity-80 mb-1">当前价格</div>
                <div class="flex items-baseline gap-2">
                    <span class="text-4xl font-bold">¥{productData.data.currentPrice.price}</span>
                    <span class="text-sm opacity-80">
                        {productData.data.currentPrice.inStock ? '有货' : '无货'}
                    </span>
                </div>
                <div class="text-sm mt-2 opacity-80">
                    店铺: {productData.data.currentPrice.shopName}
                </div>
            </div>

            <div class="grid grid-cols-3 gap-3">
                <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                    <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">最低价</div>
                    <div class="text-xl font-bold text-green-600 dark:text-green-400">
                        ¥{productData.data.stats.minPrice}
                    </div>
                </div>
                <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                    <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">最高价</div>
                    <div class="text-xl font-bold text-red-600 dark:text-red-400">
                        ¥{productData.data.stats.maxPrice}
                    </div>
                </div>
                <div class="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                    <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">平均价</div>
                    <div class="text-xl font-bold text-blue-600 dark:text-blue-400">
                        ¥{parseFloat(productData.data.stats.avgPrice).toFixed(2)}
                    </div>
                </div>
            </div>

            <div class="flex-1">
                <h3 class="text-sm font-semibold text-gray-600 dark:text-gray-400 mb-3">
                    价格历史 ({productData.data.stats.recordCount} 条记录)
                </h3>
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
                    <div class="max-h-48 overflow-auto">
                        {#if productData.data.priceHistory.length > 0}
                            <table class="w-full text-sm">
                                <thead class="bg-gray-100 dark:bg-gray-700 sticky top-0">
                                    <tr>
                                        <th class="px-4 py-2 text-left">价格</th>
                                        <th class="px-4 py-2 text-left">时间</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each productData.data.priceHistory as record, index}
                                        <tr
                                            class="border-t border-gray-200 dark:border-gray-700"
                                        >
                                            <td class="px-4 py-2 font-mono font-semibold">
                                                ¥{record.price}
                                            </td>
                                            <td class="px-4 py-2 text-gray-500">
                                                {formatDate(record.createdAt)}
                                            </td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        {:else}
                            <div class="p-4 text-center text-gray-500">
                                暂无价格历史记录
                            </div>
                        {/if}
                    </div>
                </div>
            </div>
        </div>
    {/if}
</div>
