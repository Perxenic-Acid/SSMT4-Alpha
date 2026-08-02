import { reactive } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { open } from '@tauri-apps/plugin-dialog';
import { ModManager } from '../../store/ModManager';
import { ModTagStore } from '../../store/ModTagStore';
import { ModStateStore } from '../../store/ModStateStore';
import type { Ref } from 'vue';
import type { GroupInfo, ModInfo } from './ModsManagement.types';
import { getGroupParent, getModGroupFromRelativePath, isSameOrChildPath, replacePathPrefix, replaceFsPathSegment } from './ModsManagement.paths';

type Translate = (key: string, params?: Record<string, unknown>) => string;

interface UseModsManagementGroupsOptions {
    selectedGame: Ref<string>;
    groups: Ref<GroupInfo[]>;
    mods: Ref<ModInfo[]>;
    t: Translate;
    suppressFsRefresh: (ms?: number) => void;
    syncTagStateAfterMutation: (opts?: { reloadAllMods?: boolean }) => Promise<void>;
    withOrderPersistencePaused: <T>(task: () => Promise<T>) => Promise<T>;
    loadGroupMods: (groupId: string, opts?: any) => Promise<void>;
    bumpGroupTree: () => void;
    refreshAfterMutation: (opts?: { structural?: boolean; full?: boolean }) => Promise<void>;
    groupIconVersion: Ref<number>;
    expandedKeys: Ref<string[]>;
    groupListRef: Ref<HTMLElement | null>;
    groupTreeRef: Ref<any>;
    selectedGroup: Ref<string>;
    sidebarSelectedGroup: Ref<string>;
    groupNavigationHistory: Ref<string[]>;
    groupNavigationForwardHistory: Ref<string[]>;
    isApplyingGroupHistory: Ref<boolean>;
    groupLoadToken: { value: number };
    sanitizeExpanded: (game: string) => string[];
    applyExpandedToTree: () => Promise<void>;
    persistExpandedState: () => void;
    expandedState: Ref<Record<string, string[]>>;
    sortGroupsByOrder: (game: string, parentId: string, groups: GroupInfo[]) => GroupInfo[];
    ROOT_PARENT_ID: string;
    updateAvailableGroups: (groups: GroupInfo[]) => void;
    MODS_TREE_ROOT_ID: string;
    getSubgroupPreviewUrl: (group: GroupInfo) => string;
    getGroupIconUrl: (iconPath: string) => string;
    convertFileSrc: (path: string) => string;
}

export const useModsManagementGroups = (options: UseModsManagementGroupsOptions) => {
    const ROOT_GROUP_ID = 'Root';

    // Sub Group Dialog
    const subGroupDialog = reactive({ visible: false, parentId: '', name: '', icon: '' });

    const openSubGroupDialog = (parentId: string) => {
        subGroupDialog.visible = true; subGroupDialog.parentId = parentId; subGroupDialog.name = ''; subGroupDialog.icon = '';
    };

    const pickSubGroupIcon = async () => {
        const picked = await open({ multiple: false, filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp'] }] });
        if (picked) subGroupDialog.icon = picked;
    };

    const confirmSubGroup = async () => {
        if (!subGroupDialog.name) { ElMessage.warning(options.t('modsManagement.messages.enterSubcategoryName')); return; }
        const newGroupPath = subGroupDialog.parentId ? `${subGroupDialog.parentId}/${subGroupDialog.name}` : subGroupDialog.name;
        try {
            await ModManager.createModGroup(options.selectedGame.value, newGroupPath);
            if (subGroupDialog.icon) {
                try { await ModManager.setModGroupIcon(options.selectedGame.value, newGroupPath, subGroupDialog.icon); }
                catch (e: unknown) { ElMessage.warning(options.t('modsManagement.messages.subcategoryIconSetFailed', { error: String(e) })); }
            }
            ElMessage.success(options.t('modsManagement.messages.subcategoryCreated'));
            subGroupDialog.visible = false;
            await options.refreshAfterMutation({ structural: true, full: true });
        } catch (e: unknown) { ElMessage.error(options.t('modsManagement.messages.createFailed', { error: String(e) })); }
    };

    // Group CRUD
    const createNewGroup = async () => {
        try {
            const result = await ElMessageBox.prompt(options.t('modsManagement.messages.enterNewCategoryName'), options.t('modsManagement.dialog.createCategoryTitle'), { confirmButtonText: options.t('modsManagement.common.create'), cancelButtonText: options.t('modsManagement.common.cancel') }) as { value: string };
            if (result.value) { await ModManager.createModGroup(options.selectedGame.value, result.value); ElMessage.success(options.t('modsManagement.messages.categoryCreated')); await options.refreshAfterMutation({ structural: true, full: true }); }
        } catch { /* cancelled */ }
    };

    const toggleGroup = async (group: GroupInfo) => {
        if (!group.enabled) {
            const parentId = getGroupParent(group.id);
            if (parentId && parentId !== 'Root' && parentId !== 'All') {
                const parentGroup = options.groups.value.find(g => g.id === parentId);
                if (parentGroup && !parentGroup.enabled) { ElMessage.warning(options.t('modsManagement.messages.cannotEnableCategoryWithDisabledParent', { category: group.name, parentCategory: parentGroup.name })); return; }
            }
        }
        try {
            const nextEnabled = !group.enabled;
            const oldGroupId = group.id;
            const oldPhysicalPath = group.path || group.id;
            const shouldReloadCurrentView = isSameOrChildPath(options.selectedGroup.value, oldGroupId);
            options.suppressFsRefresh(2500);
            const newPhysicalPath = await ModManager.toggleGroup(options.selectedGame.value, oldPhysicalPath, nextEnabled);
            await ModTagStore.remapPrefix(options.selectedGame.value, oldPhysicalPath, newPhysicalPath);
            await options.syncTagStateAfterMutation({ reloadAllMods: false });
            // migrate local state
            options.mods.value = options.mods.value.map((mod) => {
                const affectsRelativePath = isSameOrChildPath(mod.relativePath, oldPhysicalPath);
                const affectsId = isSameOrChildPath(mod.id, oldPhysicalPath);
                if (!affectsRelativePath && !affectsId) return mod;
                const nrp = affectsRelativePath ? replacePathPrefix(mod.relativePath, oldPhysicalPath, newPhysicalPath) : mod.relativePath;
                return { ...mod, id: affectsId ? replacePathPrefix(mod.id, oldPhysicalPath, newPhysicalPath) : mod.id, relativePath: nrp, group: getModGroupFromRelativePath(nrp), path: replaceFsPathSegment(mod.path, mod.relativePath, nrp) };
            });
            group.enabled = nextEnabled;
            group.path = newPhysicalPath;
            if (shouldReloadCurrentView) { await options.withOrderPersistencePaused(async () => { await options.loadGroupMods(options.selectedGroup.value, { showProgress: false }); }); }
            ElMessage.success(nextEnabled ? options.t('modsManagement.messages.categoryEnabled') : options.t('modsManagement.messages.categoryDisabled'));
        } catch (e: unknown) { ElMessage.error(options.t('modsManagement.messages.toggleStatusFailed', { error: String(e) })); await options.refreshAfterMutation({ structural: true, full: true }); }
    };

    const renameGroup = async (oldPath: string) => {
        const parts = oldPath.split('/'); const currentName = parts[parts.length - 1]; const parent = parts.length > 1 ? parts.slice(0, -1).join('/') : '';
        try {
            const result = await ElMessageBox.prompt(options.t('modsManagement.messages.enterNewCategoryName'), options.t('modsManagement.dialog.renameCategoryTitle'), { confirmButtonText: options.t('modsManagement.common.confirm'), cancelButtonText: options.t('modsManagement.common.cancel'), inputValue: currentName, inputPattern: /^[^\\/:*?"<>|]+$/, inputErrorMessage: options.t('modsManagement.messages.nameContainsInvalidCharacters') }) as { value: string };
            if (result.value && result.value !== currentName) {
                const newPath = parent ? `${parent}/${result.value}` : result.value;
                await ModManager.renameGroup(options.selectedGame.value, oldPath, newPath);
                options.suppressFsRefresh(1400);
                await ModTagStore.remapPrefix(options.selectedGame.value, oldPath, newPath);
                await ModTagStore.remapGroupPrefix(options.selectedGame.value, oldPath, newPath);
                await ModStateStore.remapPrefix(options.selectedGame.value, oldPath, newPath);
                await options.syncTagStateAfterMutation({ reloadAllMods: false });
                ElMessage.success(options.t('modsManagement.messages.categoryRenamed'));
                if (options.selectedGroup.value === oldPath) options.selectedGroup.value = newPath;
                if (options.sidebarSelectedGroup.value === oldPath) options.sidebarSelectedGroup.value = newPath;
                await options.refreshAfterMutation({ structural: true, full: true });
            }
        } catch (e: unknown) { if (e !== 'cancel' && (e as any)?.action !== 'cancel') { ElMessage.error(options.t('modsManagement.messages.renameFailed', { error: String(e) })); } }
    };

    const deleteGroup = async (groupName: string) => {
        try {
            await ElMessageBox.confirm(options.t('modsManagement.messages.deleteCategoryConfirm', { category: groupName }), options.t('modsManagement.dialog.deleteCategoryTitle'), { confirmButtonText: options.t('modsManagement.common.delete'), cancelButtonText: options.t('modsManagement.common.cancel'), type: 'warning' });
            await ModManager.deleteGroup(options.selectedGame.value, groupName);
            options.suppressFsRefresh(1400);
            await ModTagStore.deletePrefixMappings(options.selectedGame.value, groupName);
            await options.syncTagStateAfterMutation({ reloadAllMods: false });
            ElMessage.success(options.t('modsManagement.messages.categoryDeleted'));
            if (options.selectedGroup.value === groupName) options.selectedGroup.value = ROOT_GROUP_ID;
            if (options.sidebarSelectedGroup.value === groupName) options.sidebarSelectedGroup.value = ROOT_GROUP_ID;
            await options.refreshAfterMutation({ structural: true, full: true });
        } catch (e: unknown) { if (e !== 'cancel') { ElMessage.error(options.t('modsManagement.messages.deleteFailed', { error: String(e) })); } }
    };

    const setGroupIcon = async (groupPath: string) => {
        try {
            const selected = await open({ multiple: false, filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp'] }] });
            if (selected) { await ModManager.setModGroupIcon(options.selectedGame.value, groupPath, selected); ElMessage.success(options.t('modsManagement.messages.iconSetSuccessfully')); await options.refreshAfterMutation({ structural: true, full: true }); }
        } catch (e: unknown) { ElMessage.error(options.t('modsManagement.messages.setIconFailed', { error: String(e) })); }
    };

    const openModGroupFolder = async (groupPath: string) => {
        try { await ModManager.openModGroupFolder(options.selectedGame.value, groupPath); }
        catch (e: unknown) { ElMessage.error(options.t('modsManagement.messages.openFolderFailed', { error: String(e) })); }
    };

    // Tree navigation
    const syncTreeCurrentSelection = (groupId: string) => {
        options.sidebarSelectedGroup.value = groupId;
        const tree = options.groupTreeRef.value as { setCurrentKey: (key?: string) => void } | null;
        if (tree && typeof tree.setCurrentKey === 'function') tree.setCurrentKey(groupId === 'Root' ? undefined : groupId);
    };

    const rememberGroupNavigation = (fromGroup: string, toGroup: string) => {
        if (options.isApplyingGroupHistory.value || !fromGroup || fromGroup === toGroup) return;
        options.groupNavigationHistory.value = [...options.groupNavigationHistory.value.slice(-49), fromGroup];
        options.groupNavigationForwardHistory.value = [];
    };

    const handleGroupClick = async (data: GroupInfo, opts?: { ensureExpanded?: boolean; syncSidebar?: boolean }) => {
        if (!data?.id) return;
        const nextGroup = data.id === options.MODS_TREE_ROOT_ID ? 'Root' : data.id;
        rememberGroupNavigation(options.selectedGroup.value, nextGroup);
        options.selectedGroup.value = nextGroup;
        if (opts?.syncSidebar !== false) syncTreeCurrentSelection(options.selectedGroup.value);
        await options.loadGroupMods(options.selectedGroup.value);
    };

    const collapseSelectedGroup = async () => {
        const game = options.selectedGame.value; const currentGroup = String(options.sidebarSelectedGroup.value || '');
        if (!game || !currentGroup || currentGroup === 'All' || currentGroup === 'Root') return;
        const nextExpandedKeys = options.expandedKeys.value.filter((key) => !isSameOrChildPath(key, currentGroup));
        if (nextExpandedKeys.length === options.expandedKeys.value.length) return;
        options.expandedKeys.value = nextExpandedKeys;
        options.expandedState.value[game] = [...nextExpandedKeys];
        options.persistExpandedState();
        await options.applyExpandedToTree();
    };

    const navigateToParentGroup = async () => {
        const current = options.selectedGroup.value;
        if (!current || current === 'All' || current === 'Root') return false;
        const parent = getGroupParent(current);
        const targetGroup = parent === options.ROOT_PARENT_ID ? 'Root' : parent;
        rememberGroupNavigation(current, targetGroup);
        options.selectedGroup.value = targetGroup;
        syncTreeCurrentSelection(targetGroup);
        await options.loadGroupMods(targetGroup, { showProgress: false, refresh: true });
        return true;
    };

    const navigateBackGroup = async () => {
        const targetGroup = options.groupNavigationHistory.value.pop(); if (!targetGroup) return;
        options.isApplyingGroupHistory.value = true;
        try {
            const currentGroup = options.selectedGroup.value;
            if (currentGroup && currentGroup !== targetGroup) options.groupNavigationForwardHistory.value = [...options.groupNavigationForwardHistory.value.slice(-49), currentGroup];
            options.selectedGroup.value = targetGroup; syncTreeCurrentSelection(targetGroup);
            await options.loadGroupMods(targetGroup, { showProgress: false, refresh: true });
        } finally { options.isApplyingGroupHistory.value = false; }
    };

    const navigateForwardGroup = async () => {
        const targetGroup = options.groupNavigationForwardHistory.value.pop(); if (!targetGroup) return;
        options.isApplyingGroupHistory.value = true;
        try {
            const currentGroup = options.selectedGroup.value;
            if (currentGroup && currentGroup !== targetGroup) options.groupNavigationHistory.value = [...options.groupNavigationHistory.value.slice(-49), currentGroup];
            options.selectedGroup.value = targetGroup; syncTreeCurrentSelection(targetGroup);
            await options.loadGroupMods(targetGroup, { showProgress: false, refresh: true });
        } finally { options.isApplyingGroupHistory.value = false; }
    };

    const onMouseSideBack = (e: MouseEvent) => {
        if (e.button !== 3) return;
        if (!(e.target as HTMLElement)?.closest('.mod-manager')) return;
        e.preventDefault(); e.stopPropagation(); void navigateBackGroup();
    };

    return {
        subGroupDialog, openSubGroupDialog, pickSubGroupIcon, confirmSubGroup,
        createNewGroup, toggleGroup, renameGroup, deleteGroup, setGroupIcon, openModGroupFolder,
        handleGroupClick, collapseSelectedGroup,
        navigateToParentGroup, navigateBackGroup, navigateForwardGroup, onMouseSideBack,
        syncTreeCurrentSelection, rememberGroupNavigation,
    };
};
