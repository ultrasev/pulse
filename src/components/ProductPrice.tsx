import { useState, useEffect } from 'react';
import { fetch } from '@tauri-apps/plugin-http';

interface PriceRecord {
    price: string;
    createdAt: string;
}

interface Product {
    id: string;
    name: string;
    searchKeyword: string;
    jdSkuId: string;
    jdUrl: string;
    variant: string;
    imageUrl: string;
    shopName: string;
    isActive: boolean;
    createdAt: string;
    updatedAt: string;
}

interface CurrentPrice {
    price: string;
    shopName: string;
    inStock: boolean;
    updatedAt: string;
}

interface Stats {
    minPrice: string;
    maxPrice: string;
    avgPrice: string;
    recordCount: string;
}

interface ProductData {
    success: boolean;
    data: {
        product: Product;
        currentPrice: CurrentPrice;
        priceHistory: PriceRecord[];
        stats: Stats;
    };
}

interface CacheData {
    data: ProductData;
    skuId: string;
    timestamp: number;
}

const API_BASE = 'https://gateway.ddot.cc/api/digfrog/product';
const CACHE_KEY = 'product_price_cache';
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24小时

const getCache = (sku: string): ProductData | null => {
    try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (!cached) return null;
        const parsed: CacheData = JSON.parse(cached);
        // 检查是否匹配当前 SKU 且未过期
        if (parsed.skuId === sku && Date.now() - parsed.timestamp < CACHE_DURATION) {
            return parsed.data;
        }
    } catch {
        // ignore parse errors
    }
    return null;
};

const setCache = (sku: string, data: ProductData) => {
    try {
        const cacheData: CacheData = {
            data,
            skuId: sku,
            timestamp: Date.now()
        };
        localStorage.setItem(CACHE_KEY, JSON.stringify(cacheData));
    } catch {
        // ignore storage errors
    }
};

export default function ProductPrice() {
    const [skuId, setSkuId] = useState('100209267857');
    const [inputSku, setInputSku] = useState('100209267857');
    const [productData, setProductData] = useState<ProductData | null>(() => getCache('100209267857'));
    const [loading, setLoading] = useState(false);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [lastUpdate, setLastUpdate] = useState<number | null>(() => {
        try {
            const cached = localStorage.getItem(CACHE_KEY);
            if (!cached) return null;
            const parsed: CacheData = JSON.parse(cached);
            if (parsed.skuId === '100209267857') {
                return parsed.timestamp;
            }
        } catch {
            // ignore
        }
        return null;
    });

    const fetchProductPrice = async (sku: string, isBackground = false) => {
        if (!isBackground) {
            setLoading(true);
        } else {
            setIsRefreshing(true);
        }
        setError(null);
        try {
            const response = await fetch(`${API_BASE}/${sku}`, {
                method: 'GET',
                headers: {
                    'Accept': 'application/json'
                }
            });
            if (!response.ok) {
                throw new Error(`请求失败: ${response.status}`);
            }
            const data: ProductData = await response.json();
            if (!data.success) {
                throw new Error('API 返回失败');
            }
            setProductData(data);
            setSkuId(sku);
            setCache(sku, data);
            setLastUpdate(Date.now());
        } catch (e) {
            console.error("Failed to fetch product info:", e);
            setError(e instanceof Error ? e.message : '加载商品信息失败');
        } finally {
            if (!isBackground) {
                setLoading(false);
            } else {
                setIsRefreshing(false);
            }
        }
    };

    useEffect(() => {
        // 如果有缓存数据，后台刷新；否则立即加载
        if (productData) {
            fetchProductPrice(skuId, true);
        } else {
            fetchProductPrice(skuId);
        }
    }, []);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (inputSku.trim()) {
            const sku = inputSku.trim();
            // 先尝试从缓存加载
            const cached = getCache(sku);
            if (cached) {
                setProductData(cached);
                setSkuId(sku);
                try {
                    const cacheData = JSON.parse(localStorage.getItem(CACHE_KEY) || '{}');
                    setLastUpdate(cacheData.timestamp || Date.now());
                } catch {
                    // ignore
                }
            }
            // 然后拉取最新数据
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

    const getTimeAgo = (timestamp: number) => {
        const seconds = Math.floor((Date.now() - timestamp) / 1000);
        if (seconds < 60) return '刚刚';
        if (seconds < 3600) return `${Math.floor(seconds / 60)}分钟前`;
        if (seconds < 86400) return `${Math.floor(seconds / 3600)}小时前`;
        return `${Math.floor(seconds / 86400)}天前`;
    };

    return (
        <div className="flex flex-col gap-4 flex-1">
            {/* 搜索栏 */}
            <form onSubmit={handleSubmit} className="flex gap-2">
                <input
                    type="text"
                    value={inputSku}
                    onChange={(e) => setInputSku(e.target.value)}
                    placeholder="输入京东 SKU ID"
                    className="flex-1 px-4 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500"
                />
                <button
                    type="submit"
                    disabled={loading}
                    className="px-6 py-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
                >
                    {loading ? '查询中...' : '查询'}
                </button>
            </form>

            {/* 缓存提示 */}
            {lastUpdate && !loading && (
                <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 px-2">
                    <span>更新于 {getTimeAgo(lastUpdate)}</span>
                    {isRefreshing && <span className="text-orange-500">更新中...</span>}
                </div>
            )}

            {loading && !productData && (
                <div className="flex items-center justify-center h-64">
                    <div className="animate-spin rounded-full h-12 w-12 border-4 border-orange-500 border-t-transparent"></div>
                </div>
            )}

            {error && (
                <div className="flex flex-col items-center justify-center h-64 gap-4">
                    <div className="text-red-500 text-lg">{error}</div>
                    <button
                        onClick={() => fetchProductPrice(inputSku)}
                        className="px-4 py-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors"
                    >
                        重试
                    </button>
                </div>
            )}

            {productData && !loading && (
                <div className="flex flex-col gap-4 flex-1 overflow-auto">
                    {/* 商品基本信息 */}
                    <div className="flex gap-4 p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                        {productData.data.product.imageUrl && (
                            <img
                                src={productData.data.product.imageUrl}
                                alt={productData.data.product.name}
                                className="w-24 h-24 object-contain rounded-lg"
                            />
                        )}
                        <div className="flex-1">
                            <h2 className="text-xl font-bold mb-1">{productData.data.product.name}</h2>
                            <div className="text-sm text-gray-500 dark:text-gray-400 mb-2">
                                {productData.data.product.variant}
                            </div>
                            <a
                                href={productData.data.product.jdUrl}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-sm text-orange-500 hover:underline"
                            >
                                查看京东商品页 →
                            </a>
                        </div>
                    </div>

                    {/* 当前价格 */}
                    <div className="p-6 bg-orange-500 rounded-2xl shadow-lg text-white">
                        <div className="text-sm opacity-80 mb-1">当前价格</div>
                        <div className="flex items-baseline gap-2">
                            <span className="text-4xl font-bold">¥{productData.data.currentPrice.price}</span>
                            <span className="text-sm opacity-80">
                                {productData.data.currentPrice.inStock ? '有货' : '无货'}
                            </span>
                        </div>
                        <div className="text-sm mt-2 opacity-80">
                            店铺: {productData.data.currentPrice.shopName}
                        </div>
                    </div>

                    {/* 价格统计 */}
                    <div className="grid grid-cols-3 gap-3">
                        <div className="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">最低价</div>
                            <div className="text-xl font-bold text-green-600 dark:text-green-400">
                                ¥{productData.data.stats.minPrice}
                            </div>
                        </div>
                        <div className="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">最高价</div>
                            <div className="text-xl font-bold text-red-600 dark:text-red-400">
                                ¥{productData.data.stats.maxPrice}
                            </div>
                        </div>
                        <div className="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">平均价</div>
                            <div className="text-xl font-bold text-blue-600 dark:text-blue-400">
                                ¥{parseFloat(productData.data.stats.avgPrice).toFixed(2)}
                            </div>
                        </div>
                    </div>

                    {/* 价格历史 */}
                    <div className="flex-1">
                        <h3 className="text-sm font-semibold text-gray-600 dark:text-gray-400 mb-3">
                            价格历史 ({productData.data.stats.recordCount} 条记录)
                        </h3>
                        <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
                            <div className="max-h-48 overflow-auto">
                                {productData.data.priceHistory.length > 0 ? (
                                    <table className="w-full text-sm">
                                        <thead className="bg-gray-100 dark:bg-gray-700 sticky top-0">
                                            <tr>
                                                <th className="px-4 py-2 text-left">价格</th>
                                                <th className="px-4 py-2 text-left">时间</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {productData.data.priceHistory.map((record, index) => (
                                                <tr
                                                    key={index}
                                                    className="border-t border-gray-200 dark:border-gray-700"
                                                >
                                                    <td className="px-4 py-2 font-mono font-semibold">
                                                        ¥{record.price}
                                                    </td>
                                                    <td className="px-4 py-2 text-gray-500">
                                                        {formatDate(record.createdAt)}
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                ) : (
                                    <div className="p-4 text-center text-gray-500">
                                        暂无价格历史记录
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
