<script lang="ts">
    import { onMount } from "svelte";
    import * as mijiaService from "../lib/services/mijia";

    type SpeakerStatus = mijiaService.SpeakerStatus;

    let speaker: SpeakerStatus | null = null;
    let playbackState: string | null = null;
    let loading = false;
    let isRefreshing = false;
    let error: string | null = null;

    // Operation state
    let isOperating = false;

    // Volume control
    let volumeInput: string = "50";
    let pendingVolume: number = 50;
    let isSettingVolume = false;

    // Radio URL input
    let radioUrl: string = "";
    let isPlayingRadio = false;

    const fetchStatus = async (isBackground = false) => {
        if (!isBackground) loading = true;
        else isRefreshing = true;
        error = null;

        try {
            const [status, state] = await Promise.all([
                mijiaService.getSpeakerStatus(),
                mijiaService.getPlaybackState(),
            ]);

            speaker = status;
            playbackState = state;
            if (status) {
                volumeInput = String(status.volume);
                pendingVolume = status.volume;
            }
        } catch (e) {
            console.error("Failed to fetch speaker status:", e);
            error = "Failed to load speaker status";
        } finally {
            if (!isBackground) loading = false;
            else isRefreshing = false;
        }
    };

    const handlePlay = async () => {
        if (isOperating) return;
        isOperating = true;
        const success = await mijiaService.speakerPlay();
        if (success) {
            // Wait a bit for the state to update
            await new Promise(r => setTimeout(r, 300));
            await fetchStatus(true);
        }
        isOperating = false;
    };

    const handlePause = async () => {
        if (isOperating) return;
        isOperating = true;
        const success = await mijiaService.speakerPause();
        if (success) {
            // Wait a bit for the state to update
            await new Promise(r => setTimeout(r, 300));
            await fetchStatus(true);
        }
        isOperating = false;
    };

    const handleNext = async () => {
        if (isOperating) return;
        isOperating = true;
        const success = await mijiaService.speakerNext();
        isOperating = false;
        if (success) await fetchStatus(true);
    };

    const handlePrevious = async () => {
        if (isOperating) return;
        isOperating = true;
        const success = await mijiaService.speakerPrevious();
        isOperating = false;
        if (success) await fetchStatus(true);
    };

    const handleSetVolume = async () => {
        const vol = Number(volumeInput);
        if (isNaN(vol) || vol < 5 || vol > 100) return;

        isSettingVolume = true;
        const success = await mijiaService.setVolume(vol);
        isSettingVolume = false;

        if (success) {
            pendingVolume = vol;
            await fetchStatus(true);
        }
    };

    const handleVolumeSliderChange = async (e: Event) => {
        const input = e.target as HTMLInputElement;
        const vol = Number(input.value);
        pendingVolume = vol;
        volumeInput = String(vol);

        // Debounce: only set after user stops sliding
        if (volumeTimer) clearTimeout(volumeTimer);
        volumeTimer = setTimeout(async () => {
            const success = await mijiaService.setVolume(vol);
            if (success) await fetchStatus(true);
            else volumeInput = String(speaker?.volume ?? 50);
        }, 300);
    };

    let volumeTimer: ReturnType<typeof setTimeout> | null = null;

    const toggleMute = async () => {
        if (!speaker) return;
        const newMute = !speaker.mute;
        const success = await mijiaService.setMute(newMute);
        if (success) await fetchStatus(true);
    };

    const toggleSleepMode = async () => {
        if (!speaker) return;
        const newMode = !speaker.sleepMode;
        const success = await mijiaService.setSleepMode(newMode);
        if (success) await fetchStatus(true);
    };

    const handlePlayRadio = async () => {
        if (!radioUrl.trim()) return;
        isPlayingRadio = true;
        const success = await mijiaService.speakerPlayRadio(radioUrl);
        isPlayingRadio = false;
        if (success) {
            radioUrl = "";
            await fetchStatus(true);
        }
    };

    const handleStopAlarm = async () => {
        if (isOperating) return;
        isOperating = true;
        const success = await mijiaService.speakerStopAlarm();
        isOperating = false;
        if (success) await fetchStatus(true);
    };

    // Reactive state for UI
    $: displayStatus = (() => {
        if (isOperating) return 'Buffering...';
        if (!playbackState) return 'Unknown';
        const state = playbackState.toLowerCase();
        switch (state) {
            case 'playing': return 'Playing';
            case 'paused': return 'Paused';
            case 'stopped': return 'Stopped';
            case 'buffering': return 'Buffering...';
            default: return playbackState;
        }
    })();

    $: isCurrentlyPlaying = playbackState?.toLowerCase() === 'playing';

    onMount(() => {
        fetchStatus();
    });
</script>

{#if loading && !speaker}
    <div class="flex items-center justify-center h-64">
        <div class="animate-spin rounded-full h-12 w-12 border-4 border-orange-500 border-t-transparent"></div>
    </div>
{:else if error && !speaker}
    <div class="flex flex-col items-center justify-center h-64 gap-4">
        <div class="text-red-500 text-lg">{error}</div>
        <button
            on:click={() => fetchStatus()}
            class="px-4 py-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors"
        >
            Retry
        </button>
    </div>
{:else}
    <div class="flex flex-col gap-4 flex-1 overflow-y-auto pointer-events-auto">
        <!-- 状态卡片 -->
        <div class="relative overflow-hidden rounded-2xl shadow-lg bg-gradient-to-br from-orange-500 to-orange-600">
            <div class="relative p-6">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-xs text-white/70 mb-1 tracking-wide">XIAOMI SPEAKER</div>
                        <div class="text-2xl font-bold text-white">小爱音箱</div>
                        <div class="flex items-center gap-2 mt-2">
                            <span class="px-2 py-1 bg-white/20 rounded text-xs text-white">
                                {playbackState || 'Unknown'}
                            </span>
                            {#if speaker?.sleepMode}
                                <span class="px-2 py-1 bg-purple-500/50 rounded text-xs text-white">睡眠模式</span>
                            {/if}
                        </div>
                    </div>
                    <div class="text-5xl">🔊</div>
                </div>
            </div>
        </div>

        <!-- 播放控制 -->
        <div class="p-5 bg-white rounded-2xl shadow-sm border border-gray-200">
            <div class="flex items-center gap-2 mb-4">
                <div class="p-2 bg-orange-100 rounded-lg">
                    <svg class="w-5 h-5 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                    </svg>
                </div>
                <span class="text-sm font-medium text-gray-700">Playback</span>
                <span class="ml-auto text-sm font-semibold {isCurrentlyPlaying ? 'text-green-600' : 'text-gray-500'}">{displayStatus}</span>
            </div>

            <div class="flex items-center justify-center gap-4">
                <button
                    on:click={handlePrevious}
                    disabled={isOperating}
                    class="p-3 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed rounded-xl transition-all hover:scale-105"
                    title="Previous"
                >
                    <svg class="w-6 h-6 text-gray-700" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/>
                    </svg>
                </button>

                {#if isCurrentlyPlaying}
                    <button
                        on:click={handlePause}
                        disabled={isOperating}
                        class="p-4 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed rounded-xl transition-all hover:scale-105 flex items-center gap-2"
                        title="Pause"
                    >
                        {#if isOperating}
                            <div class="animate-spin rounded-full h-5 w-5 border-2 border-gray-600 border-t-transparent"></div>
                        {:else}
                            <svg class="w-6 h-6 text-gray-700" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
                            </svg>
                        {/if}
                    </button>
                {:else}
                    <button
                        on:click={handlePlay}
                        disabled={isOperating}
                        class="p-4 bg-orange-500 hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-xl transition-all hover:scale-105 shadow-lg flex items-center gap-2"
                        title="Play"
                    >
                        {#if isOperating}
                            <div class="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"></div>
                        {:else}
                            <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M8 5v14l11-7z"/>
                            </svg>
                        {/if}
                    </button>
                {/if}

                <button
                    on:click={handleNext}
                    disabled={isOperating}
                    class="p-3 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed rounded-xl transition-all hover:scale-105"
                    title="Next"
                >
                    <svg class="w-6 h-6 text-gray-700" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
                    </svg>
                </button>
            </div>
        </div>

        <!-- 音量控制 -->
        <div class="p-5 bg-white rounded-2xl shadow-sm border border-gray-200">
            <div class="flex items-center gap-2 mb-4">
                <div class="p-2 bg-blue-100 rounded-lg">
                    <svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"></path>
                    </svg>
                </div>
                <span class="text-sm font-medium text-gray-700">音量</span>
                <span class="ml-auto text-sm font-bold text-blue-600">{pendingVolume}%</span>
            </div>

            <!-- 静音开关 -->
            <button
                on:click={toggleMute}
                class="w-full flex items-center justify-between p-3 mb-3 bg-gray-100 hover:bg-gray-200 rounded-xl transition-all"
            >
                <span class="text-sm text-gray-700">
                    {speaker?.mute ? '🔇 已静音' : '🔊 正常'}
                </span>
                <div class="w-12 h-6 rounded-full transition-colors {speaker?.mute ? 'bg-gray-400' : 'bg-blue-500'}">
                    <div class="w-5 h-5 bg-white rounded-full shadow transform transition-transform {speaker?.mute ? 'translate-x-0' : 'translate-x-6'}"></div>
                </div>
            </button>

            <!-- 音量滑块 -->
            <div class="space-y-3">
                <input
                    type="range"
                    min="5"
                    max="100"
                    value={pendingVolume}
                    on:input={handleVolumeSliderChange}
                    disabled={speaker?.mute || isSettingVolume}
                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-500 disabled:opacity-50"
                />

                <div class="flex gap-2">
                    <input
                        type="number"
                        min="5"
                        max="100"
                        bind:value={volumeInput}
                        class="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-center focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button
                        on:click={handleSetVolume}
                        disabled={isSettingVolume}
                        class="px-4 py-2 bg-blue-500 hover:bg-blue-600 disabled:opacity-50 text-white rounded-lg transition-colors text-sm font-medium"
                    >
                        {isSettingVolume ? '设置中...' : '设置'}
                    </button>
                </div>
            </div>
        </div>

        <!-- 播放网络广播 -->
        <div class="p-5 bg-white rounded-2xl shadow-sm border border-gray-200">
            <div class="flex items-center gap-2 mb-4">
                <div class="p-2 bg-green-100 rounded-lg">
                    <svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                    </svg>
                </div>
                <span class="text-sm font-medium text-gray-700">网络广播</span>
            </div>

            <div class="flex gap-2">
                <input
                    type="text"
                    bind:value={radioUrl}
                    placeholder="输入音频 URL"
                    class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-green-500"
                />
                <button
                    on:click={handlePlayRadio}
                    disabled={isPlayingRadio || !radioUrl.trim()}
                    class="px-4 py-2 bg-green-500 hover:bg-green-600 disabled:opacity-50 text-white rounded-lg transition-colors text-sm font-medium"
                >
                    {isPlayingRadio ? '播放中...' : '播放'}
                </button>
            </div>
        </div>

        <!-- 其他设置 -->
        <div class="grid grid-cols-2 gap-3">
            <button
                on:click={toggleSleepMode}
                class="p-4 bg-white rounded-2xl shadow-sm border border-gray-200 hover:shadow-md hover:border-purple-300 transition-all"
            >
                <div class="text-2xl mb-2">🌙</div>
                <div class="text-sm font-medium text-gray-700">睡眠模式</div>
                <div class="text-xs text-gray-400 mt-1">{speaker?.sleepMode ? '已开启' : '已关闭'}</div>
            </button>

            <button
                on:click={handleStopAlarm}
                class="p-4 bg-white rounded-2xl shadow-sm border border-gray-200 hover:shadow-md hover:border-red-300 transition-all"
            >
                <div class="text-2xl mb-2">⏰</div>
                <div class="text-sm font-medium text-gray-700">停止闹钟</div>
                <div class="text-xs text-gray-400 mt-1">停止响铃</div>
            </button>
        </div>

        <!-- 刷新按钮 -->
        <button
            on:click={() => fetchStatus()}
            disabled={isRefreshing}
            class="w-full py-3 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 rounded-2xl transition-all font-medium flex items-center justify-center gap-2"
        >
            {#if isRefreshing}
                <div class="animate-spin rounded-full h-5 w-5 border-2 border-gray-500 border-t-transparent"></div>
                <span>刷新中...</span>
            {:else}
                <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                </svg>
                <span>Refresh</span>
            {/if}
        </button>
    </div>
{/if}
