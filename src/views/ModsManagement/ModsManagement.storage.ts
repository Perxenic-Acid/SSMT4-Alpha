import { debugLog } from '../../utils/debugLog';

const DEFAULT_SIDEBAR_WIDTH = 220;

export const GROUP_ORDER_KEY = 'ssmt4_group_orders_v1';
export const GROUP_EXPANDED_KEY = 'ssmt4_group_expanded_v1';
export const ORDER_STORAGE_KEY = 'ssmt4_mod_manual_orders_v1';
export const SELECTED_GROUP_KEY = 'ssmt4_mod_selected_group_v1';
export const SIDEBAR_WIDTH_KEY = 'ssmt4_mod_sidebar_width_v1';
export const SUBGROUP_PREVIEW_CACHE_KEY = 'ssmt4_subgroup_preview_cache_v1';
export const SIDEBAR_MIN_WIDTH = 150;
export const SIDEBAR_MAX_WIDTH = 500;
export const SUBGROUP_PREVIEW_MAX_IMAGES = 8;
export const SUBGROUP_PREVIEW_MAX_GROUPS = 400;

const readJsonStorage = <T>(key: string, fallback: T): T => {
    if (typeof localStorage === 'undefined') {
        return fallback;
    }

    try {
        const raw = localStorage.getItem(key);
        if (!raw) {
            return fallback;
        }

        const parsed = JSON.parse(raw);
        if (parsed && typeof parsed === 'object') {
            return parsed as T;
        }
    } catch (error) {
        console.warn(`Failed to load ${key}`, error);
    }

    return fallback;
};

export const loadSidebarWidth = () => {
    if (typeof localStorage === 'undefined') return DEFAULT_SIDEBAR_WIDTH;
    try {
        const raw = localStorage.getItem(SIDEBAR_WIDTH_KEY);
        if (!raw) return DEFAULT_SIDEBAR_WIDTH;
        const value = Number(raw);
        if (!Number.isFinite(value)) return DEFAULT_SIDEBAR_WIDTH;
        return Math.min(SIDEBAR_MAX_WIDTH, Math.max(SIDEBAR_MIN_WIDTH, value));
    } catch {
        return DEFAULT_SIDEBAR_WIDTH;
    }
};

export const persistSidebarWidth = (width: number) => {
    if (typeof localStorage === 'undefined') return;
    try {
        localStorage.setItem(SIDEBAR_WIDTH_KEY, String(Math.round(width)));
    } catch (error) {
        console.warn('Failed to save sidebar width', error);
    }
};

export const loadSelectedGroupState = () => readJsonStorage<Record<string, string>>(SELECTED_GROUP_KEY, {});

export const persistSelectedGroupState = (value: Record<string, string>) => {
    if (typeof localStorage === 'undefined') return;
    try {
        localStorage.setItem(SELECTED_GROUP_KEY, JSON.stringify(value));
    } catch (error) {
        console.warn('Failed to save selected group state', error);
    }
};

export const loadManualOrders = () => readJsonStorage<Record<string, Record<string, string[]>>>(ORDER_STORAGE_KEY, {});

export const persistManualOrders = (value: Record<string, Record<string, string[]>>) => {
    if (typeof localStorage === 'undefined') return;
    try {
        localStorage.setItem(ORDER_STORAGE_KEY, JSON.stringify(value));
    } catch (error) {
        console.warn('Failed to save manual orders', error);
    }
};

export const loadGroupOrders = () => readJsonStorage<Record<string, Record<string, string[]>>>(GROUP_ORDER_KEY, {});

export const persistGroupOrders = (value: Record<string, Record<string, string[]>>) => {
    if (typeof localStorage === 'undefined') return;
    try {
        localStorage.setItem(GROUP_ORDER_KEY, JSON.stringify(value));
    } catch (error) {
        console.warn('Failed to save group orders', error);
    }
};

export const loadExpandedState = () => {
    const value = readJsonStorage<Record<string, string[]>>(GROUP_EXPANDED_KEY, {});
    if (Object.keys(value).length === 0) {
        debugLog('GroupExpanded', 'No stored data, using empty state');
    } else {
        debugLog('GroupExpanded', 'Loaded from storage', value);
    }
    return value;
};

export const persistExpandedState = (value: Record<string, string[]>) => {
    if (typeof localStorage === 'undefined') return;
    try {
        localStorage.setItem(GROUP_EXPANDED_KEY, JSON.stringify(value));
        debugLog('GroupExpanded', 'Saved', value);
    } catch (error) {
        console.warn('Failed to save expanded state', error);
    }
};

export const loadSubgroupPreviewCache = () => {
    const parsed = readJsonStorage<Record<string, unknown>>(SUBGROUP_PREVIEW_CACHE_KEY, {});
    const normalized: Record<string, string[]> = {};

    Object.entries(parsed).forEach(([key, value]) => {
        if (Array.isArray(value)) {
            normalized[key] = value
                .filter((item): item is string => typeof item === 'string' && item.length > 0)
                .slice(0, SUBGROUP_PREVIEW_MAX_IMAGES);
        }
    });

    return normalized;
};

export const persistSubgroupPreviewCache = (value: Record<string, string[]>) => {
    if (typeof localStorage === 'undefined') return;
    try {
        const entries = Object.entries(value);
        const trimmed = entries
            .slice(Math.max(0, entries.length - SUBGROUP_PREVIEW_MAX_GROUPS))
            .reduce((acc, [key, images]) => {
                acc[key] = (images || []).slice(0, SUBGROUP_PREVIEW_MAX_IMAGES);
                return acc;
            }, {} as Record<string, string[]>);
        localStorage.setItem(SUBGROUP_PREVIEW_CACHE_KEY, JSON.stringify(trimmed));
    } catch (error) {
        console.warn('Failed to persist subgroup preview cache', error);
    }
};