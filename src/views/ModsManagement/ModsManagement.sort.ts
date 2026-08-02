import type { ModInfo } from './ModsManagement.types';

export type SortCriterion = 'name' | 'modified' | 'tag' | 'manual';
export type SortOrder = 'asc' | 'desc';

interface ModSortOptions {
    criterion: SortCriterion;
    order: SortOrder;
    sortTagId: string;
    manualOrderIds: string[];
    getManualOrderId: (mod: ModInfo) => string;
    getTagIds: (mod: ModInfo) => string[];
    getTagNames: (mod: ModInfo) => string[];
}

const compareNames = (a: string, b: string) => a.localeCompare(b, undefined, {
    numeric: true,
    sensitivity: 'base',
});

export const compareModsByDefault = (a: ModInfo, b: ModInfo) => {
    const modifiedComparison = (b.lastModified || 0) - (a.lastModified || 0);
    return modifiedComparison || compareNames(a.name, b.name);
};

export const buildModSortComparator = (options: ModSortOptions): ((a: ModInfo, b: ModInfo) => number) => {
    const direction = options.order === 'asc' ? 1 : -1;

    switch (options.criterion) {
        case 'name':
            return (a, b) => direction * compareNames(a.name, b.name);
        case 'modified':
            return (a, b) => {
                const modifiedComparison = (b.lastModified || 0) - (a.lastModified || 0);
                return direction * (modifiedComparison || compareNames(a.name, b.name));
            };
        case 'tag':
            return (a, b) => {
                if (options.sortTagId) {
                    const aHasTag = options.getTagIds(a).includes(options.sortTagId);
                    const bHasTag = options.getTagIds(b).includes(options.sortTagId);
                    if (aHasTag !== bHasTag) {
                        return direction * (aHasTag ? -1 : 1);
                    }
                } else {
                    const aTagKey = options.getTagNames(a).sort(compareNames).join('\u0000');
                    const bTagKey = options.getTagNames(b).sort(compareNames).join('\u0000');
                    if (aTagKey !== bTagKey) {
                        if (!aTagKey) return direction;
                        if (!bTagKey) return -direction;
                        return direction * compareNames(aTagKey, bTagKey);
                    }
                }
                return direction * compareNames(a.name, b.name);
            };
        case 'manual':
        default: {
            const orderMap = new Map(options.manualOrderIds.map((id, index) => [id, index]));
            return (a, b) => {
                const aIndex = orderMap.get(options.getManualOrderId(a));
                const bIndex = orderMap.get(options.getManualOrderId(b));
                if (aIndex !== undefined && bIndex !== undefined) return aIndex - bIndex;
                if (aIndex !== undefined) return -1;
                if (bIndex !== undefined) return 1;
                return compareModsByDefault(a, b);
            };
        }
    }
};
