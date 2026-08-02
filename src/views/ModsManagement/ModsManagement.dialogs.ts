import { reactive, ref, nextTick } from 'vue';
import { ElMessage } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
import { open } from '@tauri-apps/plugin-dialog';
import { ModManager } from '../../store/ModManager';
import { calculateContextMenuPosition } from '../../utils/ContextMenuPosition';
import type { Ref } from 'vue';
import type { ArchiveExportFormat, GroupInfo, ModInfo } from './ModsManagement.types';

type Translate = (key: string, params?: Record<string, unknown>) => string;

interface UseModsManagementDialogsOptions {
    selectedGame: Ref<string>;
    t: Translate;
}

export const useModsManagementDialogs = (options: UseModsManagementDialogsOptions) => {
    // Context Menu State
    const contextMenu = reactive({ visible: false, x: 0, y: 0, target: null as ModInfo | null });
    const groupContextMenu = reactive({ visible: false, x: 0, y: 0, target: null as GroupInfo | null });
    const contextMenuRef = ref<HTMLElement | null>(null);

    const closeContextMenu = () => { contextMenu.visible = false; };
    const closeGroupContextMenu = () => { groupContextMenu.visible = false; };

    const adjustContextMenuPosition = (clientX: number, clientY: number) => {
        const menuEl = contextMenuRef.value;
        if (!menuEl) return;
        const menuRect = menuEl.getBoundingClientRect();
        const pos = calculateContextMenuPosition({ clientX, clientY, menuWidth: menuRect.width, menuHeight: menuRect.height });
        contextMenu.x = pos.x;
        contextMenu.y = pos.y;
    };

    const openContextMenu = (clientX: number, clientY: number, target: ModInfo) => {
        // Close first so that re-opening on a different target triggers watchers
        if (contextMenu.visible) {
            contextMenu.visible = false;
        }
        groupContextMenu.visible = false;
        nextTick(() => {
            contextMenu.visible = true;
            contextMenu.target = target;
            contextMenu.x = clientX;
            contextMenu.y = clientY;
            nextTick(() => { adjustContextMenuPosition(clientX, clientY); });
        });
    };

    const openGroupContextMenu = (clientX: number, clientY: number, target: GroupInfo) => {
        // Close first so that re-opening on a different target triggers watchers
        if (groupContextMenu.visible) {
            groupContextMenu.visible = false;
        }
        contextMenu.visible = false;
        nextTick(() => {
            groupContextMenu.visible = true;
            groupContextMenu.target = target;
            groupContextMenu.x = clientX;
            groupContextMenu.y = clientY;
        });
    };

    const showModContextMenu = (e: MouseEvent, mod: ModInfo) => { openContextMenu(e.clientX, e.clientY, mod); };
    const showGroupContextMenu = (e: MouseEvent, group: GroupInfo) => {
        if (group.id === 'All' || group.id === 'Root') return;
        openGroupContextMenu(e.clientX, e.clientY, group);
    };

    // Export Archive Dialog
    const archiveExportFormats: ArchiveExportFormat[] = ['zip', '7z', 'rar'];
    const exportArchiveDialog = reactive({
        visible: false, exporting: false, modName: '', modPath: '', modRelativePath: '',
        archiveName: '', outputDir: '', password: '', format: 'zip' as ArchiveExportFormat,
    });

    const isValidArchiveFileName = (value: string) => {
        const trimmed = value.trim();
        return !!trimmed && /^[^\\/:*?"<>|]+$/.test(trimmed) && !trimmed.endsWith('.') && !trimmed.endsWith(' ');
    };

    const openExportArchiveDialog = (mod: ModInfo) => {
        exportArchiveDialog.visible = true;
        exportArchiveDialog.exporting = false;
        exportArchiveDialog.modName = mod.name;
        exportArchiveDialog.modPath = mod.path;
        exportArchiveDialog.modRelativePath = mod.relativePath;
        exportArchiveDialog.archiveName = mod.name.replace(/[\\/:*?"<>|]/g, '_').trim() || 'Mod';
        exportArchiveDialog.outputDir = '';
        exportArchiveDialog.password = '';
        exportArchiveDialog.format = 'zip';
    };

    const cancelExportArchive = () => {
        if (exportArchiveDialog.exporting) return;
        exportArchiveDialog.visible = false;
    };

    const chooseExportOutputDir = async () => {
        try {
            const selected = await open({ directory: true, multiple: false, title: options.t('modsManagement.dialog.selectExportFolderTitle') });
            if (selected && typeof selected === 'string') { exportArchiveDialog.outputDir = selected; return selected; }
        } catch (error) { ElMessage.error(options.t('modsManagement.messages.selectExportFolderFailed', { error: String(error) })); }
        return '';
    };

    const confirmExportArchive = async () => {
        if (exportArchiveDialog.exporting) return;
        const archiveName = exportArchiveDialog.archiveName.trim();
        if (!isValidArchiveFileName(archiveName)) { ElMessage.warning(options.t('modsManagement.messages.nameContainsInvalidCharacters')); return; }
        let outputDir = exportArchiveDialog.outputDir.trim();
        if (!outputDir) { outputDir = await chooseExportOutputDir(); if (!outputDir) return; }
        exportArchiveDialog.exporting = true;
        try {
            const installDir = await ModManager.getInstallDir(options.selectedGame.value);
            const outputPath = await invoke<string>('export_mod_archive', {
                installDir, modRelativePath: exportArchiveDialog.modRelativePath, outputDir,
                archiveName, format: exportArchiveDialog.format, password: exportArchiveDialog.password.trim() || null,
            });
            ElMessage.success(options.t('modsManagement.messages.exportArchiveSuccess', { path: outputPath }));
            exportArchiveDialog.visible = false;
            try { await revealItemInDir(outputPath); } catch { await openPath(outputDir); }
        } catch (error) {
            ElMessage.error(options.t('modsManagement.messages.exportArchiveFailed', { error: String(error) }));
            if (String(error).includes('Mod list may be stale')) { /* handled by caller */ }
        } finally { exportArchiveDialog.exporting = false; }
    };

    // Mod Key List Dialog
    const modKeyListDialog = reactive({ visible: false, modId: '' });
    const closeModKeyList = () => { modKeyListDialog.visible = false; modKeyListDialog.modId = ''; };

    return {
        contextMenu, groupContextMenu, contextMenuRef,
        closeContextMenu, closeGroupContextMenu,
        openContextMenu, openGroupContextMenu,
        showModContextMenu, showGroupContextMenu,
        archiveExportFormats, exportArchiveDialog,
        openExportArchiveDialog, cancelExportArchive, chooseExportOutputDir, confirmExportArchive,
        modKeyListDialog, closeModKeyList,
    };
};
