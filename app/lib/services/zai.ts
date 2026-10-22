import { fetch } from '@tauri-apps/plugin-http';

const API_URL = 'https://api.z.ai/api/monitor/usage/quota/limit';
const BEARER_KEY = 'zai_bearer_token';
const REFRESH_INTERVAL = 5 * 60 * 1000;

export interface ZaiQuota {
    percentage: number;
    resetInMinutes: number | null;
    resetAt: string | null;
}

interface ZaiLimit {
    type: string;
    percentage?: number;
    nextResetTime?: number;
}

interface ZaiResponse {
    data?: {
        limits?: ZaiLimit[];
    };
}

export const getBearer = (): string => {
    if (typeof localStorage === 'undefined') return '';
    return localStorage.getItem(BEARER_KEY) ?? '';
};

export const setBearer = (token: string) => localStorage.setItem(BEARER_KEY, token);

export const hasBearer = (): boolean => Boolean(getBearer());

export const fetchZaiQuota = async (): Promise<ZaiQuota> => {
    const bearer = getBearer();
    if (!bearer) throw new Error('未配置 Bearer Token');

    const response = await fetch(API_URL, {
        method: 'GET',
        headers: {
            'Accept': 'application/json',
            'Authorization': `Bearer ${bearer}`
        }
    });

    if (!response.ok) {
        throw new Error(`请求失败: ${response.status}`);
    }

    const json = await response.json() as ZaiResponse;
    const tokensLimit = json.data?.limits?.find(l => l.type === 'TOKENS_LIMIT');

    if (!tokensLimit || tokensLimit.percentage == null) {
        throw new Error('响应中无 TOKENS_LIMIT 数据');
    }

    let resetInMinutes: number | null = null;
    let resetAt: string | null = null;
    if (tokensLimit.nextResetTime) {
        const sec = Math.max(0, Math.floor((tokensLimit.nextResetTime - Date.now()) / 1000));
        resetInMinutes = Math.floor(sec / 60);
        resetAt = new Date(tokensLimit.nextResetTime).toLocaleString('sv-SE', {
            year: 'numeric', month: '2-digit', day: '2-digit',
            hour: '2-digit', minute: '2-digit', second: '2-digit'
        });
    }

    return {
        percentage: tokensLimit.percentage,
        resetInMinutes,
        resetAt
    };
};

export const shouldRefresh = (lastFetch: number): boolean =>
    Date.now() - lastFetch > REFRESH_INTERVAL;
