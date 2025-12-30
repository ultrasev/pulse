import { useState, useEffect } from 'react';
import { fetch } from '@tauri-apps/plugin-http';

interface IpData {
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

const CACHE_KEY = 'ip_info_cache';
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24小时

const getCache = (): IpData | null => {
    try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (!cached) return null;
        const parsed: CacheData = JSON.parse(cached);
        // 检查是否未过期
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

const getTimeAgo = (timestamp: number) => {
    const seconds = Math.floor((Date.now() - timestamp) / 1000);
    if (seconds < 60) return '刚刚';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}分钟前`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}小时前`;
    return `${Math.floor(seconds / 86400)}天前`;
};

export default function IpInfo() {
    const [ipData, setIpData] = useState<IpData | null>(() => getCache());
    const [loading, setLoading] = useState(false);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [lastUpdate, setLastUpdate] = useState<number | null>(() => {
        try {
            const cached = localStorage.getItem(CACHE_KEY);
            if (!cached) return null;
            const parsed: CacheData = JSON.parse(cached);
            return parsed.timestamp;
        } catch {
            return null;
        }
    });

    const fetchIpInfo = async (isBackground = false) => {
        if (!isBackground) {
            setLoading(true);
        } else {
            setIsRefreshing(true);
        }
        setError(null);
        try {
            const response = await fetch('https://cufo.cc', {
                method: 'GET',
                headers: {
                    'Accept': 'application/json'
                }
            });
            if (!response.ok) {
                throw new Error('Failed to fetch IP info');
            }
            const data = await response.json();
            setIpData(data);
            setCache(data);
            setLastUpdate(Date.now());
        } catch (e) {
            console.error("Failed to fetch IP info:", e);
            setError('Failed to load IP information');
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
        if (ipData) {
            fetchIpInfo(true);
        } else {
            fetchIpInfo();
        }
    }, []);

    // 国家代码转换为 emoji 旗帜
    const countryToFlag = (countryCode: string) => {
        const codePoints = countryCode
            .toUpperCase()
            .split('')
            .map(char => 127397 + char.charCodeAt(0));
        return String.fromCodePoint(...codePoints);
    };

    if (loading && !ipData) {
        return (
            <div className="flex items-center justify-center h-64">
                <div className="animate-spin rounded-full h-12 w-12 border-4 border-cyan-500 border-t-transparent"></div>
            </div>
        );
    }

    if (error && !ipData) {
        return (
            <div className="flex flex-col items-center justify-center h-64 gap-4">
                <div className="text-red-500 text-lg">{error}</div>
                <button
                    onClick={() => fetchIpInfo()}
                    className="px-4 py-2 bg-cyan-500 text-white rounded-lg hover:bg-cyan-600 transition-colors"
                >
                    Retry
                </button>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-4 flex-1">
            {/* IP 地址主卡片 */}
            <div className="p-6 bg-gradient-to-br from-cyan-500 to-blue-600 rounded-2xl shadow-lg text-white">
                <div className="text-sm opacity-80 mb-1">Public IP Address</div>
                <div className="text-3xl font-mono font-bold tracking-wider">{ipData?.ip}</div>
            </div>

            {/* 缓存提示 */}
            {lastUpdate && !loading && (
                <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 px-2">
                    <span>更新于 {getTimeAgo(lastUpdate)}</span>
                    {isRefreshing && <span className="text-cyan-500">更新中...</span>}
                </div>
            )}

            {/* 详情网格 */}
            <div className="grid grid-cols-2 gap-3 flex-1">
                {/* 位置信息 */}
                <div className="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                    <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">Location</div>
                    <div className="text-lg font-semibold flex items-center gap-2">
                        {ipData?.country && <span className="text-2xl">{countryToFlag(ipData.country)}</span>}
                        <span>{ipData?.city}, {ipData?.country}</span>
                    </div>
                    <div className="text-sm text-gray-500">{ipData?.region}</div>
                </div>

                {/* ISP 信息 */}
                <div className="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                    <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">ISP</div>
                    <div className="text-sm font-semibold line-clamp-2">{ipData?.ISP}</div>
                    <div className="text-sm text-gray-500 mt-1">ASN: {ipData?.ASN}</div>
                </div>

                {/* 坐标 */}
                <div className="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                    <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">Coordinates</div>
                    <div className="font-mono text-sm">
                        <div>Lat: {ipData?.latitude}</div>
                        <div>Lon: {ipData?.longitude}</div>
                    </div>
                </div>

                {/* 时区 */}
                <div className="p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                    <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">Timezone</div>
                    <div className="text-lg font-semibold">{ipData?.timezone}</div>
                    <div className="text-sm text-gray-500">Postal: {ipData?.postalCode || 'N/A'}</div>
                </div>
            </div>

            {/* 刷新按钮 */}
            <button
                onClick={() => fetchIpInfo()}
                className="w-full py-3 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-xl transition-colors font-medium"
            >
                ↻ Refresh
            </button>
        </div>
    );
}
