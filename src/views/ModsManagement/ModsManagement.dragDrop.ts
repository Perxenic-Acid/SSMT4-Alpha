import { ref, reactive } from 'vue';
import { ElMessage } from 'element-plus';
import { ModManager } from '../../store/ModManager';
import { ModTagStore } from '../../store/ModTagStore';
import { ModStateStore } from '../../store/ModStateStore';
import { debugLog, debugWarn } from '../../utils/debugLog';
import type { Ref } from 'vue';
import type { GroupInfo, ModInfo } from './ModsManagement.types';
import { buildMovedModRelativePath as buildMovedModRelativePathUtil, getGroupParent as getGroupParentUtil } from './ModsManagement.paths';

type Translate = (key: string, params?: Record<string, unknown>) => string;

interface UseModsManagementDragDropOptions {
    selectedGame: Ref<string>;
    mods: Ref<ModInfo[]>;
    availableGroups: Ref<GroupInfo[]>;
    t: Translate;
    suppressFsRefresh: (ms?: number) => void;
    syncTagStateAfterMutation: (opts?: { reloadAllMods?: boolean }) => Promise<void>;
    refreshAfterMutation: (opts?: { structural?: boolean; full?: boolean }) => Promise<void>;
    allModsCatalogLoadedForGame: Ref<string>;
    moveModToGroup: (mod: ModInfo, groupName: string) => Promise<void>;
    applyManualReorder: (dragId: string, targetId: string) => void;
    reorderGroup: (sourceId: string, targetId: string) => void;
    MODS_TREE_ROOT_ID: string;
}

export const useModsManagementDragDrop = (opts: UseModsManagementDragDropOptions) => {
    const buildMovedModRelativePath = (mod: ModInfo, targetGroupId: string) => buildMovedModRelativePathUtil(mod, targetGroupId);
    const getGroupParent = (id: string) => getGroupParentUtil(id);
    const resolveGroupPhysicalPath = (groupId: string) => {
        if (!groupId || groupId === 'Root' || groupId === 'All' || groupId === opts.MODS_TREE_ROOT_ID) {
            return 'Root';
        }

        return opts.availableGroups.value.find((group) => group.id === groupId)?.path || groupId;
    };

    // Native DnD + Manual fallback
    const draggingMod = ref<ModInfo | null>(null);
    const draggingOrderId = ref<string | null>(null);
    const dragOverId = ref<string | null>(null);

    const manualSortState = reactive({ active: false, startX: 0, startY: 0, hasMoved: false, mod: null as ModInfo | null });
    let manualSortGroupHover: HTMLElement | null = null;

    // Group drag state
    const groupDragState = reactive({ active: false, startX: 0, startY: 0, hasMoved: false, sourceId: '' as string, targetId: null as string | null, sourceParent: '' as string });
    const groupHoverId = ref<string | null>(null);

    // --- Native DnD handlers ---
    const onDragEnter = (e: DragEvent) => {
        debugLog('DragEnter', e.currentTarget);
        (e.currentTarget as HTMLElement).classList.add('drag-over');
    };

    const onDragOver = (e: DragEvent) => {
        e.preventDefault();
        if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
        const target = e.currentTarget as HTMLElement;
        if (!target.classList.contains('drag-over')) target.classList.add('drag-over');
    };

    const onDragLeave = (e: DragEvent) => {
        const target = e.currentTarget as HTMLElement;
        const related = e.relatedTarget as Node | null;
        if (target.contains(related)) return;
        target.classList.remove('drag-over');
    };

    const onDrop = async (e: DragEvent, targetGroupId: string) => {
        e.preventDefault();
        const target = e.currentTarget as HTMLElement;
        target.classList.remove('drag-over');
        const normalizedTargetGroupId = targetGroupId === opts.MODS_TREE_ROOT_ID ? 'Root' : targetGroupId;
        const rawData = e.dataTransfer?.getData('text/plain');
        const mod = draggingMod.value;
        debugLog('Drop', { targetGroupId: normalizedTargetGroupId, rawData, modId: mod?.id });
        if (mod && (mod.id === rawData || !rawData)) {
            if (mod.group === normalizedTargetGroupId) return;
            if (normalizedTargetGroupId === 'All') return;
            try {
                const targetPhysicalPath = resolveGroupPhysicalPath(normalizedTargetGroupId);
                const nextRelativePath = buildMovedModRelativePath(mod, targetPhysicalPath);
                await ModManager.moveModToGroup(opts.selectedGame.value, mod.relativePath, targetPhysicalPath);
                opts.suppressFsRefresh(1400);
                await ModTagStore.remapModPath(opts.selectedGame.value, mod.relativePath, nextRelativePath);
                await ModStateStore.remapModPath(opts.selectedGame.value, mod.relativePath, nextRelativePath);
                await opts.syncTagStateAfterMutation({ reloadAllMods: opts.allModsCatalogLoadedForGame.value === opts.selectedGame.value });
                ElMessage.success({
                    message: opts.t('modsManagement.messages.movedToGroup', {
                        group: normalizedTargetGroupId === 'Root' ? opts.t('modsManagement.ui.uncategorizedRoot') : normalizedTargetGroupId,
                    }), offset: 48
                });
                await opts.refreshAfterMutation();
            } catch (e: unknown) {
                ElMessage.error({ message: opts.t('modsManagement.messages.moveFailed', { error: String(e) }), offset: 48 });
            } finally { draggingMod.value = null; document.body.style.userSelect = ''; }
        } else { debugWarn('Drop', 'No mod captured or ID mismatch', { rawData, dragging: mod?.id }); }
    };

    // --- Manual sort (mod reorder) ---
    const onCardMouseDownWrapper = (e: MouseEvent, mod: ModInfo) => { onManualSortMouseDown(e, mod); };

    const onManualSortMouseDown = (e: MouseEvent, mod: ModInfo) => {
        if (e.button !== 0) return;
        manualSortState.active = true;
        manualSortState.startX = e.clientX; manualSortState.startY = e.clientY;
        manualSortState.hasMoved = false; manualSortState.mod = mod;
        draggingOrderId.value = mod.id;
        document.addEventListener('mousemove', onManualSortMouseMove);
        document.addEventListener('mouseup', onManualSortMouseUp);
    };

    const setManualSortGroupHover = (el: HTMLElement | null) => {
        if (manualSortGroupHover && manualSortGroupHover !== el) manualSortGroupHover.classList.remove('drag-over');
        if (el && manualSortGroupHover !== el) el.classList.add('drag-over');
        manualSortGroupHover = el;
    };

    const onManualSortMouseMove = (e: MouseEvent) => {
        if (!manualSortState.active || !manualSortState.mod) return;
        const dx = e.clientX - manualSortState.startX;
        const dy = e.clientY - manualSortState.startY;
        if (!manualSortState.hasMoved && Math.hypot(dx, dy) > 3) {
            manualSortState.hasMoved = true;
            document.body.style.userSelect = 'none';
        }
        if (manualSortState.hasMoved) {
            const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
            const groupEl = el?.closest?.('[data-group-id]') as HTMLElement | null;
            if (groupEl) { setManualSortGroupHover(groupEl); dragOverId.value = null; }
            else {
                setManualSortGroupHover(null);
                const card = el?.closest?.('.mod-card') as HTMLElement | null;
                dragOverId.value = card?.dataset?.modId || null;
            }
        }
    };

    const resetManualSortState = () => {
        manualSortState.active = false; manualSortState.hasMoved = false; manualSortState.mod = null;
        draggingOrderId.value = null; dragOverId.value = null;
        setManualSortGroupHover(null);
        document.body.style.userSelect = '';
    };

    const onManualSortMouseUp = (e: MouseEvent) => {
        document.removeEventListener('mousemove', onManualSortMouseMove);
        document.removeEventListener('mouseup', onManualSortMouseUp);
        if (!manualSortState.active || !manualSortState.mod) { resetManualSortState(); return; }
        if (manualSortState.hasMoved) {
            const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
            const groupEl = el?.closest?.('[data-group-id]') as HTMLElement | null;
            const targetGroupId = groupEl?.dataset.groupId;
            if (targetGroupId && targetGroupId !== 'All' && manualSortState.mod.group !== targetGroupId) {
                opts.moveModToGroup(manualSortState.mod, targetGroupId);
                resetManualSortState();
                return;
            }
            const card = el?.closest?.('.mod-card') as HTMLElement | null;
            const targetId = card?.dataset?.modId || null;
            if (targetId && targetId !== manualSortState.mod.id) {
                opts.applyManualReorder(manualSortState.mod.id, targetId);
            }
        }
        resetManualSortState();
    };

    // --- Group reorder ---
    const onGroupMouseDown = (e: MouseEvent, groupId: string) => {
        if (groupId === 'All' || groupId === 'Root' || groupId === opts.MODS_TREE_ROOT_ID) return;
        if (e.button !== 0) return;
        const target = e.target as HTMLElement | null;
        if (target?.closest('.el-tree-node__expand-icon')) return;
        groupDragState.active = true;
        groupDragState.startX = e.clientX; groupDragState.startY = e.clientY;
        groupDragState.hasMoved = false;
        groupDragState.sourceId = groupId;
        groupDragState.sourceParent = getGroupParent(groupId);
        groupDragState.targetId = null;
        document.addEventListener('mousemove', onGroupMouseMove);
        document.addEventListener('mouseup', onGroupMouseUp);
    };

    const onGroupMouseMove = (e: MouseEvent) => {
        if (!groupDragState.active) return;
        const dx = e.clientX - groupDragState.startX;
        const dy = e.clientY - groupDragState.startY;
        if (!groupDragState.hasMoved && Math.hypot(dx, dy) > 3) {
            groupDragState.hasMoved = true;
            document.body.style.userSelect = 'none';
        }
        if (groupDragState.hasMoved) {
            const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
            const node = el?.closest?.('[data-group-id]') as HTMLElement | null;
            const targetId = node?.dataset.groupId || null;
            const targetParent = node?.dataset.parentId || null;
            if (targetId && targetParent === groupDragState.sourceParent) {
                groupHoverId.value = targetId; groupDragState.targetId = targetId;
            } else { groupHoverId.value = null; groupDragState.targetId = null; }
        }
    };

    const onGroupMouseUp = () => {
        document.removeEventListener('mousemove', onGroupMouseMove);
        document.removeEventListener('mouseup', onGroupMouseUp);
        if (groupDragState.hasMoved && groupDragState.targetId) {
            opts.reorderGroup(groupDragState.sourceId, groupDragState.targetId);
        }
        resetGroupDrag();
    };

    const resetGroupDrag = () => {
        groupDragState.active = false; groupDragState.hasMoved = false;
        groupDragState.sourceId = ''; groupDragState.targetId = null; groupDragState.sourceParent = '';
        groupHoverId.value = null;
        document.body.style.userSelect = '';
    };

    return {
        draggingMod, draggingOrderId, dragOverId, manualSortState,
        groupDragState, groupHoverId,
        onDragEnter, onDragOver, onDragLeave, onDrop,
        onCardMouseDownWrapper,
        onGroupMouseDown, onGroupMouseMove, onGroupMouseUp, resetGroupDrag,
    };
};
