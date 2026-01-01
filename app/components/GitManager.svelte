<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    interface GitBranch {
        name: string;
        current: boolean;
    }

    interface GitState {
        branches: GitBranch[];
        repo_path: string;
    }

    let state: GitState | null = null;
    let loading = true;
    let error: string | null = null;
    let switching = false;
    // To optimistically update UI while switching
    let pendingBranch: string | null = null;

    const fetchBranches = async () => {
        // Only show full loading state on initial load
        if (!state) loading = true;
        error = null;
        try {
            state = await invoke<GitState>('get_git_branches');
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    };

    const switchBranch = async (branchName: string) => {
        if (switching || (state && state.branches.find(b => b.name === branchName)?.current)) return;

        switching = true;
        pendingBranch = branchName;

        try {
            await invoke('switch_git_branch', { branch: branchName });
            await fetchBranches();
        } catch (e) {
            error = String(e);
        } finally {
            switching = false;
            pendingBranch = null;
        }
    };

    onMount(() => {
        fetchBranches();
    });
</script>

<div class="h-full flex flex-col gap-4">
    <div class="p-4 bg-gray-100 h-full flex flex-col rounded-xl overflow-hidden">
        <h2 class="text-sm uppercase text-gray-500 font-semibold mb-2 ml-4 tracking-wider">Claude Models</h2>

        {#if loading && !state}
            <div class="flex items-center justify-center h-40">
                <div class="text-gray-500 animate-pulse">Loading settings...</div>
            </div>
        {:else if error}
            <div class="p-4 mx-4 bg-red-50 border border-red-200 rounded-lg">
                <div class="text-red-600 font-mono text-sm break-all mb-2">
                    {error}
                </div>
                <button
                    on:click={fetchBranches}
                    class="px-4 py-2 bg-red-100 text-red-700 rounded-md hover:bg-red-200 transition-colors text-sm font-medium"
                >
                    Retry
                </button>
            </div>
        {:else if state}
            <div class="flex-1 overflow-y-auto">
                <div class="bg-white rounded-xl overflow-hidden shadow-sm">
                    {#each state.branches as branch, i}
                        <!-- Row -->
                        <button
                            class="w-full flex items-center justify-between p-4 bg-white transition-colors
                                   border-b border-gray-100 last:border-0
                                   hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                            on:click={() => switchBranch(branch.name)}
                            disabled={switching}
                        >
                            <!-- Left: Label -->
                            <span class="text-base font-medium text-gray-900">
                                {branch.name}
                            </span>

                            <!-- Right: Toggle Switch -->
                            <!--
                                Logic:
                                - Checked if it's the current branch OR if it's the one we are pending switch to.
                                - If we are pending switch to X, current branch Y should visually turn off (optional, but cleaner).
                            -->
                            <div
                                class="relative inline-flex h-7 w-12 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-indigo-600 focus-visible:ring-offset-2
                                {(branch.current && pendingBranch === null) || pendingBranch === branch.name
                                    ? 'bg-green-500'
                                    : 'bg-gray-200'}"
                            >
                                <span class="sr-only">Use setting</span>
                                <span
                                    aria-hidden="true"
                                    class="pointer-events-none inline-block h-6 w-6 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out
                                    {(branch.current && pendingBranch === null) || pendingBranch === branch.name
                                        ? 'translate-x-5'
                                        : 'translate-x-0'}"
                                ></span>
                            </div>
                        </button>
                    {/each}
                </div>

                <div class="mt-4 px-4 text-xs text-gray-400 font-mono text-center">
                    Repository: {state.repo_path}
                </div>
            </div>
        {/if}
    </div>
</div>
