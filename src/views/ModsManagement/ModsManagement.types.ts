import type { ModInfo as MMModInfo, GroupInfo as MMGroupInfo, ModKeyInfo as MMModKeyInfo } from '../../store/ModManager';
import type { ModTagDefinition } from '../../store/ModTagStore';

export type ModInfo = MMModInfo;
export type GroupInfo = MMGroupInfo;
export type ModKeyInfo = MMModKeyInfo;

export interface ArchivePreview {
    root_dirs: string[];
    file_count: number;
    has_ini: boolean;
    format: string;
}

export interface TagManagementDialogState {
    visible: boolean;
    editingId: string;
    name: string;
    color: string;
    iconSourcePath: string;
    removeIcon: boolean;
    saving: boolean;
}

export interface ModTagDialogState {
    visible: boolean;
    modId: string;
    modName: string;
    selectedTagIds: string[];
    saving: boolean;
}

export interface ModKeyEditorDialogState {
    visible: boolean;
    loading: boolean;
    saving: boolean;
    modId: string;
    modName: string;
    modRelativePath: string;
    modPath: string;
    items: ModKeyInfo[];
}

export interface SubGroupDialogState {
    visible: boolean;
    parentId: string;
    name: string;
    icon: string;
}

export interface InstallFormState {
    archivePath: string;
    modName: string;
    targetGroup: string;
    password: string;
    selectedTagIds: string[];
}

export interface InstallProgressState {
    visible: boolean;
    percent: number;
    stage: string;
}

export type ArchiveExportFormat = 'zip' | '7z' | 'rar';

export interface ExportArchiveDialogState {
    visible: boolean;
    exporting: boolean;
    modName: string;
    modPath: string;
    modRelativePath: string;
    archiveName: string;
    outputDir: string;
    password: string;
    format: ArchiveExportFormat;
}

export interface ModAnalysisResult {
    modKeyList: ModKeyInfo[];
    excludedPreviewFileNames: string[];
}

export interface OrderContext {
    game: string;
    group: string;
}

export type GroupsAutocompleteFetcher = (query: string, callback: (items: Array<{ value: string }>) => void) => void;
export type TagLookupMap = Map<string, ModTagDefinition>;
