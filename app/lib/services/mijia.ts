import { invoke } from '@tauri-apps/api/core';

// Types
export interface SpeakerStatus {
    volume: number;
    mute: boolean;
    sleepMode: boolean;
}

// Speaker API - 使用 Rust commands
export async function getSpeakerStatus(): Promise<SpeakerStatus | null> {
    try {
        const props = ['volume', 'mute', 'sleep-mode'];
        const results = await Promise.all(
            props.map(async (prop) => {
                const result = await invoke<any>('get_device_prop', { prop });
                return { prop, value: result?.value };
            })
        );

        const data: Record<string, any> = {};
        results.forEach(({ prop, value }) => {
            data[prop] = value;
        });

        return {
            volume: data.volume ?? 50,
            mute: data.mute ?? false,
            sleepMode: data['sleep-mode'] ?? false,
        };
    } catch (e) {
        console.error('Failed to get speaker status:', e);
        return null;
    }
}

export async function getPlaybackState(): Promise<string | null> {
    try {
        return await invoke<string>('get_playback_state');
    } catch (e) {
        console.error('Failed to get playback state:', e);
        return null;
    }
}

export async function speakerPlay(): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('execute_device_action', {
            action: 'play',
            params: null
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Play failed:', e);
        return false;
    }
}

export async function speakerPause(): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('execute_device_action', {
            action: 'pause',
            params: null
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Pause failed:', e);
        return false;
    }
}

export async function speakerNext(): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('execute_device_action', {
            action: 'next',
            params: null
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Next failed:', e);
        return false;
    }
}

export async function speakerPrevious(): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('execute_device_action', {
            action: 'previous',
            params: null
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Previous failed:', e);
        return false;
    }
}

export async function speakerPlayRadio(url: string): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('execute_device_action', {
            action: 'play-radio',
            params: [url]
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Play radio failed:', e);
        return false;
    }
}

export async function speakerStopAlarm(): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('execute_device_action', {
            action: 'stop-alarm',
            params: null
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Stop alarm failed:', e);
        return false;
    }
}

export async function setVolume(volume: number): Promise<boolean> {
    if (volume < 5 || volume > 100) return false;
    try {
        const result = await invoke<{ success: boolean }>('set_device_prop', {
            prop: 'volume',
            value: volume
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Set volume failed:', e);
        return false;
    }
}

export async function setMute(mute: boolean): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('set_device_prop', {
            prop: 'mute',
            value: mute
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Set mute failed:', e);
        return false;
    }
}

export async function setSleepMode(sleepMode: boolean): Promise<boolean> {
    try {
        const result = await invoke<{ success: boolean }>('set_device_prop', {
            prop: 'sleep-mode',
            value: sleepMode
        });
        return result?.success ?? false;
    } catch (e) {
        console.error('Set sleep mode failed:', e);
        return false;
    }
}
