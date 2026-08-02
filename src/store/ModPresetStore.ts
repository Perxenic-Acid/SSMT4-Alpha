import { exists, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { join } from '@tauri-apps/api/path';
import { defineStore } from 'pinia';
import { ModStateStore, type RememberedModVariableState } from './ModStateStore';
import { D3dxIniManager } from './D3dxIniManager';
import { PathHelper } from '../helper/PathHelper';
import {
    buildModRuntimeRestoreValues,
    collectModRuntimeVariables,
    removeModRuntimeEntriesFromLines,
} from '../views/ModsManagement/D3dxUserIniVariables';

// ---------------------------------------------------------------------------
// Public interfaces
// ---------------------------------------------------------------------------

export interface ModPreset {
    id: string;
    name: string;
    variables: RememberedModVariableState[];
    createdAt: number;
    active: boolean;
}

export interface ModPresetsFile {
    version: number;
    presets: ModPreset[];
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PRESETS_FILE_NAME = '.ssmt-presets.json';
const PRESETS_VERSION = 1;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const generatePresetId = () => `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

const normalizePreset = (value: unknown): ModPreset | null => {
    if (!value || typeof value !== 'object') return null;
    const c = value as Partial<ModPreset>;
    const id = typeof c.id === 'string' && c.id.trim() ? c.id.trim() : generatePresetId();
    const name = typeof c.name === 'string' ? c.name.trim().slice(0, 64) : '';
    if (!name) return null;
    const variables = Array.isArray(c.variables)
        ? c.variables.filter((v): v is RememberedModVariableState =>
            !!v && typeof v === 'object' && typeof (v as RememberedModVariableState).runtimeKey === 'string' && typeof (v as RememberedModVariableState).variableName === 'string')
        : [];
    return {
        id,
        name,
        variables,
        createdAt: Number.isFinite(c.createdAt) ? Number(c.createdAt) : Date.now(),
        active: !!c.active,
    };
};

// ---------------------------------------------------------------------------
// Pinia store
// ---------------------------------------------------------------------------

export const useModPresetStore = defineStore('modPresets', () => {

    async function getPresetsPath(modPath: string): Promise<string> {
        return join(modPath, PRESETS_FILE_NAME);
    }

    async function loadPresets(modPath: string): Promise<ModPresetsFile> {
        const path = await getPresetsPath(modPath);
        if (!(await exists(path))) {
            return { version: PRESETS_VERSION, presets: [] };
        }
        try {
            const raw = await readTextFile(path);
            const parsed = JSON.parse(raw) as Partial<ModPresetsFile>;
            const presets = (Array.isArray(parsed.presets) ? parsed.presets : [])
                .map(normalizePreset)
                .filter((p): p is ModPreset => p !== null);
            // Ensure only one active
            let seenActive = false;
            for (const p of presets) {
                if (p.active) {
                    if (seenActive) p.active = false;
                    else seenActive = true;
                }
            }
            return { version: PRESETS_VERSION, presets };
        } catch {
            return { version: PRESETS_VERSION, presets: [] };
        }
    }

    async function savePresets(modPath: string, data: ModPresetsFile): Promise<void> {
        const path = await getPresetsPath(modPath);
        await writeTextFile(path, JSON.stringify(data, null, 2));
    }

    async function createPreset(modPath: string, name: string, variables: RememberedModVariableState[]): Promise<ModPreset> {
        const data = await loadPresets(modPath);
        // Deactivate all existing
        for (const p of data.presets) p.active = false;
        const preset: ModPreset = {
            id: generatePresetId(),
            name: name.trim().slice(0, 64),
            variables,
            createdAt: Date.now(),
            active: true,
        };
        data.presets.push(preset);
        await savePresets(modPath, data);
        return preset;
    }

    async function deletePreset(modPath: string, presetId: string): Promise<ModPresetsFile> {
        const data = await loadPresets(modPath);
        data.presets = data.presets.filter(p => p.id !== presetId);
        await savePresets(modPath, data);
        return data;
    }

    async function activatePreset(modPath: string, presetId: string): Promise<ModPresetsFile> {
        const data = await loadPresets(modPath);
        for (const p of data.presets) p.active = (p.id === presetId);
        await savePresets(modPath, data);
        return data;
    }

    async function renamePreset(modPath: string, presetId: string, newName: string): Promise<ModPresetsFile> {
        const data = await loadPresets(modPath);
        const preset = data.presets.find(p => p.id === presetId);
        if (preset) {
            preset.name = newName.trim().slice(0, 64);
        }
        await savePresets(modPath, data);
        return data;
    }

    async function resetPresetState(
        gameName: string,
        modRelativePath: string,
        modPath: string,
    ): Promise<ModPresetsFile> {
        const data = await loadPresets(modPath);
        for (const p of data.presets) p.active = false;

        const migotoDir = await PathHelper.GetGame3DmigotoFolderPath(gameName);
        if (migotoDir) {
            const userIniPath = await join(migotoDir, 'd3dx_user.ini');
            if (await exists(userIniPath)) {
                let lines = await D3dxIniManager.loadIni(userIniPath);
                const constantsEntries = D3dxIniManager.getSectionEntries(lines, 'Constants');
                const nextLines = removeModRuntimeEntriesFromLines(lines, modRelativePath, constantsEntries);
                if (nextLines.length !== lines.length) {
                    lines = nextLines;
                    await D3dxIniManager.saveIni(userIniPath, lines);
                }
            }

            await ModStateStore.saveModState(gameName, modRelativePath, []);
        }

        await savePresets(modPath, data);
        return data;
    }

    /**
     * Capture the current d3dx_user.ini [Constants] values for a mod
     * and save them as a new named preset.
     */
    async function captureAsPreset(
        gameName: string,
        modRelativePath: string,
        modPath: string,
        presetName: string,
    ): Promise<ModPreset | null> {
        const migotoDir = await PathHelper.GetGame3DmigotoFolderPath(gameName);
        if (!migotoDir) throw new Error('3Dmigoto install directory is not configured');

        const userIniPath = await join(migotoDir, 'd3dx_user.ini');
        if (!(await exists(userIniPath))) return null;

        const lines = await D3dxIniManager.loadIni(userIniPath);
        const constantsEntries = D3dxIniManager.getSectionEntries(lines, 'Constants');
        const variables = collectModRuntimeVariables(modRelativePath, constantsEntries);
        if (variables.length === 0) return null;

        return createPreset(modPath, presetName, variables);
    }

    /**
     * Restore a preset's values into d3dx_user.ini [Constants].
     */
    async function applyPreset(
        gameName: string,
        modRelativePath: string,
        modPath: string,
        presetId: string,
    ): Promise<boolean> {
        const data = await loadPresets(modPath);
        const preset = data.presets.find(p => p.id === presetId);
        if (!preset || preset.variables.length === 0) return false;

        const migotoDir = await PathHelper.GetGame3DmigotoFolderPath(gameName);
        if (!migotoDir) throw new Error('3Dmigoto install directory is not configured');

        const userIniPath = await join(migotoDir, 'd3dx_user.ini');
        if (!(await exists(userIniPath))) return false;

        let lines = await D3dxIniManager.loadIni(userIniPath);
        const constantsEntries = D3dxIniManager.getSectionEntries(lines, 'Constants');
        lines = removeModRuntimeEntriesFromLines(lines, modRelativePath, constantsEntries);

        const runtimeValues = buildModRuntimeRestoreValues(modRelativePath, preset.variables);
        lines = D3dxIniManager.setIniValues(lines, 'Constants', runtimeValues);
        await D3dxIniManager.saveIni(userIniPath, lines);

        await activatePreset(modPath, presetId);
        return true;
    }

    return {
        loadPresets,
        savePresets,
        createPreset,
        deletePreset,
        activatePreset,
        renamePreset,
        resetPresetState,
        captureAsPreset,
        applyPreset,
    };
});

/**
 * Backward-compatible wrapper.
 */
export const ModPresetStore = {
    get loadPresets() { return useModPresetStore().loadPresets },
    get savePresets() { return useModPresetStore().savePresets },
    get createPreset() { return useModPresetStore().createPreset },
    get deletePreset() { return useModPresetStore().deletePreset },
    get activatePreset() { return useModPresetStore().activatePreset },
    get renamePreset() { return useModPresetStore().renamePreset },
    get resetPresetState() { return useModPresetStore().resetPresetState },
    get captureAsPreset() { return useModPresetStore().captureAsPreset },
    get applyPreset() { return useModPresetStore().applyPreset },
} as const;
