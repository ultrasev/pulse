import { fetch } from '@tauri-apps/plugin-http';

const API_BASE = 'https://gateway.ddot.cc/api/digfrog/product';
const CACHE_KEY = 'product_price_cache';
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24小时

export interface PriceRecord {
    price: string;
    createdAt: string;
}

export interface Product {
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

export interface CurrentPrice {
    price: string;
    shopName: string;
    inStock: boolean;
    updatedAt: string;
}

export interface Stats {
    minPrice: string;
    maxPrice: string;
    avgPrice: string;
    recordCount: string;
}

export interface ProductData {
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

const getCache = (sku: string): ProductData | null => {
    try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (!cached) return null;
        const parsed: CacheData = JSON.parse(cached);
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
            skuId,
            timestamp: Date.now()
        };
        localStorage.setItem(CACHE_KEY, JSON.stringify(cacheData));
    } catch {
        // ignore storage errors
    }
};

export const getTimeAgo = (timestamp: number): string => {
    const seconds = Math.floor((Date.now() - timestamp) / 1000);
    if (seconds < 60) return '刚刚';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}分钟前`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}小时前`;
    return `${Math.floor(seconds / 86400)}天前`;
};

export const getCacheTimestamp = (sku: string): number | null => {
    try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (!cached) return null;
        const parsed: CacheData = JSON.parse(cached);
        if (parsed.skuId === sku) {
            return parsed.timestamp;
        }
    } catch {
        // ignore
    }
    return null;
};

export const getCached = (sku: string): ProductData | null => {
    return getCache(sku);
};

export const fetchProductPrice = async (sku: string): Promise<ProductData> => {
    const response = await fetch(`${API_BASE}/${sku}`, {
        method: 'GET',
        headers: {
            'Accept': 'application/json'
        }
    });

    if (!response.ok) {
        throw new Error(`请求失败: ${response.status}`);
    }

    const data = await response.json() as ProductData;

    if (!data.success) {
        throw new Error('API 返回失败');
    }

    setCache(sku, data);
    return data;
};
