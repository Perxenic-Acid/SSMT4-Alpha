import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { ElMessage } from 'element-plus';
import { ModManager } from '../../store/ModManager';
import type { ModScanSignal } from '../../store/ModManager';
import { migotoIniService } from './MigotoIni';
import type { Ref } from 'vue';
import type { GroupInfo, ModInfo } from './ModsManagement.types';

type Translate = (key: string, params?: Record<string, unknown>) => string;

interface UseModsManagementWatcherOptions {
    selectedGame: Ref<string>;
    selectedGroup: Ref<string>;
    sidebarSelectedGroup: Ref<string>;
    mods: Ref<ModInfo[]>;
    availableGroups: Ref<GroupInfo[]>;
    currentSubGroups: Ref<GroupInfo[]>;
    loadedModsCount: Ref<number>;
    totalModsCount: Ref<number>;
    switchingGroup: Ref<boolean>;
    suppressFsRefreshUntil: Ref<number>;
    t: Translate;
    loadTagState: (game: string) => Promise<void>;
    hasActiveTagFilter: Ref<boolean>;
    refreshAllModsCatalog: (opts?: { force?: boolean }) => Promise<void>;
    allModsCatalogLoadedForGame: Ref<string>;
    loadGroupMods: (groupId: string, opts?: any) => Promise<void>;
    appendModsIncrementally: (mods: ModInfo[], token: number) => Promise<void>;
    updateAvailableGroups: (groups: GroupInfo[]) => void;
    preloadSubgroupPreviewImages: (groups: GroupInfo[]) => Promise<void>;
    queueVisibleModAnalysis: (mods: ModInfo[]) => void;
    withOrderPersistencePaused: <T>(task: () => Promise<T>) => Promise<T>;
    invalidateSubgroupPreviewByGame: (game: string) => void;
    resolvePreferredGroup: (game: string, preferred?: string) => Promise<string>;
    sanitizeExpanded: (game: string) => string[];
    applyExpandedToTree: () => Promise<void>;
    refreshModsToken: { value: number };
    MOD_RENDER_BATCH_SIZE: number;
    groupLoadToken: { value: number };
    startWatchingToken: { value: number };
    makeScanSignal: (getToken: () => number, token: number) => ModScanSignal;
    installProgress: { visible: boolean; percent: number; stage: string };
}

export const useModsManagementWatcher = (options: UseModsManagementWatcherOptions) => {
    const makeScanSignal = options.makeScanSignal;

    const startWatching = async (gameName: string) => {
        ++options.startWatchingToken.value;
        try {
            await options.withOrderPersistencePaused(async () => {
                // Apply cached snapshot
                const { ModManager: MM } = await import('../../store/ModManager');
                const selected = options.selectedGroup.value && options.selectedGroup.value !== 'All' ? options.selectedGroup.value : 'Root';
                const rootCached = MM.getCachedGroup(gameName, 'Root');
                if (rootCached) options.updateAvailableGroups(rootCached.groups);
                const targetCached = selected === 'Root' ? rootCached : MM.getCachedGroup(gameName, selected);
                if (targetCached) {
                    options.currentSubGroups.value = targetCached.groups;
                    const token = ++options.groupLoadToken.value;
                    await options.appendModsIncrementally(targetCached.mods, token);
                }
                await refreshMods(gameName, { preserveVisible: true });
            });
            const installDir = await ModManager.getInstallDir(gameName);
            await invoke('watch_mod_library', { installDir });
        } catch (error) { console.error('Failed to start watching:', error); }
    };

    const silentRefresh = async () => {
        if (!options.selectedGame.value) return;
        const currentRefreshToken = ++options.refreshModsToken.value;
        const signal = makeScanSignal(() => options.refreshModsToken.value, currentRefreshToken);
        try {
            await options.withOrderPersistencePaused(async () => {
                ModManager.clearCache(options.selectedGame.value);
                migotoIniService.clearGame(options.selectedGame.value);
                options.invalidateSubgroupPreviewByGame(options.selectedGame.value);
                await options.loadGroupMods(options.selectedGroup.value, { signal });
            });
        } catch (e) { if (!ModManager.isScanCancelled(e)) console.error("Silent refresh failed", e); }
    };

    const refreshMods = async (gameName: string, opts?: { preserveVisible?: boolean }) => {
        const currentRefreshToken = ++options.refreshModsToken.value;
        const preserveVisible = !!opts?.preserveVisible;
        const signal = makeScanSignal(() => options.refreshModsToken.value, currentRefreshToken);
        try {
            await options.withOrderPersistencePaused(async () => {
                if (!preserveVisible) { options.availableGroups.value = []; options.mods.value = []; options.loadedModsCount.value = 0; options.totalModsCount.value = 0; }
                const streamed = await loadGroupModsStreaming(gameName, 'Root', currentRefreshToken);
                if (signal.isCancelled?.()) return;
                const nextGroup = await options.resolvePreferredGroup(gameName, options.selectedGroup.value);
                if (options.selectedGroup.value !== nextGroup) options.selectedGroup.value = nextGroup;
                if (options.sidebarSelectedGroup.value !== nextGroup) options.sidebarSelectedGroup.value = nextGroup;
                if (!streamed && options.selectedGroup.value && options.selectedGroup.value !== 'All' && options.selectedGroup.value !== 'Root') {
                    await options.loadGroupMods(options.selectedGroup.value, { showProgress: !preserveVisible, signal });
                }
            });
            if (signal.isCancelled?.()) return;
            await options.loadTagState(gameName);
            options.allModsCatalogLoadedForGame.value = '';
            if (options.hasActiveTagFilter.value) await options.refreshAllModsCatalog({ force: true });
        } catch (error) {
            if (ModManager.isScanCancelled(error)) return;
            console.error('Failed to scan mods:', error);
            ElMessage.error(options.t('modsManagement.messages.scanFailed', { error: String(error) }));
            options.mods.value = []; options.availableGroups.value = [];
        }
    };

    const loadGroupMods = async (groupId: string, opts?: { showProgress?: boolean; signal?: { isCancelled?: () => boolean }; refresh?: boolean }) => {
        if (!options.selectedGame.value) return;
        const token = ++options.groupLoadToken.value;
        const showProgress = opts?.showProgress !== false;
        const isAllGroup = groupId === 'All';
        const path = groupId === 'Root' ? 'Root' : groupId;
        const signal = { isCancelled: () => token !== options.groupLoadToken.value || !!opts?.signal?.isCancelled?.() };
        if (!opts?.refresh && !isAllGroup) {
            const cached = ModManager.getCachedGroup(options.selectedGame.value, String(path));
            if (cached) { options.updateAvailableGroups(cached.groups); options.currentSubGroups.value = cached.groups; void options.preloadSubgroupPreviewImages(cached.groups); await options.appendModsIncrementally(cached.mods, token); if (token !== options.groupLoadToken.value) return; }
        }
        if (showProgress) options.switchingGroup.value = true;
        try {
            const { mods: newMods, groups: newGroups } = isAllGroup
                ? await ModManager.refreshLibrary(options.selectedGame.value, signal)
                : await ModManager.scanGroup(options.selectedGame.value, path as string, signal, { refresh: !!opts?.refresh });
            if (token !== options.groupLoadToken.value) return;
            if (signal?.isCancelled?.()) return;
            options.updateAvailableGroups(newGroups);
            options.currentSubGroups.value = isAllGroup ? [] : newGroups;
            if (!isAllGroup) void options.preloadSubgroupPreviewImages(newGroups);
            await options.appendModsIncrementally(newMods, token);
        } catch (e) {
            if (token !== options.groupLoadToken.value) return;
            if (ModManager.isScanCancelled(e)) return;
            options.currentSubGroups.value = []; options.mods.value = []; options.loadedModsCount.value = 0; options.totalModsCount.value = 0;
            ElMessage.error(options.t('modsManagement.messages.loadModFailed', { error: String(e) }));
        } finally { if (showProgress && token === options.groupLoadToken.value) options.switchingGroup.value = false; }
    };

    const loadGroupModsStreaming = async (gameName: string, groupPath: string, refreshToken: number): Promise<boolean> => {
        const relativePath = (groupPath === 'Root' || groupPath === 'All') ? '' : groupPath;
        const installDir = await ModManager.getInstallDir(gameName);
        let streamedChunks = false;
        const accumulatedMods: ModInfo[] = []; const accumulatedGroups: GroupInfo[] = [];
        const unlistenChunk = await listen<any>('mod-library-scan-chunk', (event) => {
            const payload = event.payload; if (!payload) return;
            switch (payload.phase) {
                case 'start': options.totalModsCount.value = payload.total ?? 0; accumulatedMods.length = 0; accumulatedGroups.length = 0; options.currentSubGroups.value = []; break;
                case 'chunk': streamedChunks = true;
                    if (payload.mods?.length) { accumulatedMods.push(...payload.mods); options.mods.value = [...accumulatedMods].sort((a: ModInfo, b: ModInfo) => a.name.toLowerCase().localeCompare(b.name.toLowerCase())); options.loadedModsCount.value = options.mods.value.length; options.queueVisibleModAnalysis(payload.mods); }
                    if (payload.groups?.length) { accumulatedGroups.push(...payload.groups); options.updateAvailableGroups(payload.groups); options.currentSubGroups.value = [...accumulatedGroups].sort((a: GroupInfo, b: GroupInfo) => a.name.toLowerCase().localeCompare(b.name.toLowerCase())); }
                    break;
                case 'done': options.totalModsCount.value = payload.totalMods ?? accumulatedMods.length; break;
            }
        });
        try {
            const result = await invoke<{ mods: ModInfo[]; groups: GroupInfo[] }>('mod_library_stream_scan', { gameName, installDir, groupPath: relativePath || 'Root' });
            if (options.refreshModsToken.value !== refreshToken) return streamedChunks;
            if (result) { options.updateAvailableGroups(result.groups); options.currentSubGroups.value = result.groups; options.mods.value = result.mods; options.loadedModsCount.value = result.mods.length; options.totalModsCount.value = result.mods.length; if (result.mods.length > 0) options.queueVisibleModAnalysis(result.mods); }
        } catch (e) { if (!ModManager.isScanCancelled(e)) console.error('Stream scan failed:', e); }
        finally { unlistenChunk(); }
        return streamedChunks;
    };

    let debounceTimer: ReturnType<typeof setTimeout> | null = null;
    let unlistenFileChange: UnlistenFn | null = null;

    const setupFileWatcher = async () => {
        unlistenFileChange = await listen<string[]>('mod-library-files-changed', () => {
            if (debounceTimer) clearTimeout(debounceTimer);
            debounceTimer = setTimeout(async () => {
                if (!options.selectedGame.value) return;
                if (Date.now() < options.suppressFsRefreshUntil.value) return;
                ModManager.clearCache(options.selectedGame.value);
                migotoIniService.clearGame(options.selectedGame.value);
                options.invalidateSubgroupPreviewByGame(options.selectedGame.value);
                await loadGroupMods(options.selectedGroup.value, { showProgress: false, refresh: true });
            }, 600);
        });
    };

    const teardownFileWatcher = () => {
        if (unlistenFileChange) unlistenFileChange();
        if (debounceTimer) clearTimeout(debounceTimer);
        invoke('unwatch_mod_library').catch(e => console.error(e));
    };

    return { startWatching, silentRefresh, refreshMods, loadGroupMods, loadGroupModsStreaming, setupFileWatcher, teardownFileWatcher };
};
