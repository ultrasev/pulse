import { fetch } from '@tauri-apps/plugin-http';

const API_URL = 'https://cufo.cc';
const CACHE_KEY = 'ip_info_cache';
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24小时

export interface IpData {
    ip: string;
    ASN: number;
    ISP: string;
    publicIP: string;
    country: string;
    city: string;
    region: string;
    latitude: string;
    longitude: string;
    postalCode: string;
    timezone: string;
}

interface CacheData {
    data: IpData;
    timestamp: number;
}

const getCache = (): IpData | null => {
    try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (!cached) return null;
        const parsed: CacheData = JSON.parse(cached);
        if (Date.now() - parsed.timestamp < CACHE_DURATION) {
            return parsed.data;
        }
    } catch {
        // ignore parse errors
    }
    return null;
};

const setCache = (data: IpData) => {
    try {
        const cacheData: CacheData = {
            data,
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

export const getCacheTimestamp = (): number | null => {
    try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (!cached) return null;
        const parsed: CacheData = JSON.parse(cached);
        return parsed.timestamp;
    } catch {
        return null;
    }
};

export const getCached = (): IpData | null => {
    return getCache();
};

export const fetchIpInfo = async (): Promise<IpData> => {
    const response = await fetch(API_URL, {
        method: 'GET',
        headers: {
            'Accept': 'application/json'
        }
    });

    if (!response.ok) {
        throw new Error(`请求失败: ${response.status}`);
    }

    const data = await response.json() as IpData;
    setCache(data);
    return data;
};
