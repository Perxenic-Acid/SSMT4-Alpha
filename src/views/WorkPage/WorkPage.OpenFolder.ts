import { ElMessage } from 'element-plus';
import { join } from '@tauri-apps/api/path';
import { openPath as openExternal } from '@tauri-apps/plugin-opener';
import { mkdir, readDir } from '@tauri-apps/plugin-fs';
import { PathHelper } from '../../helper/PathHelper';
import { GlobalConfig } from '../../store/GlobalConfig';
import { i18n } from '../../i18n';

const t = i18n.global.t;

const openTargetPath = async (path: string | undefined, emptyMsg: string): Promise<void> => {
	if (!path) {
		ElMessage.warning(emptyMsg);
		return;
	}

	try {
		await mkdir(path, { recursive: true });
		await openExternal(path);
	} catch (err) {
		console.error('Failed to open path', path, err);
		ElMessage.error(t('workPage.messages.openFolderFailed'));
	}
};

const openTargetFile = async (filePath: string | undefined, emptyMsg: string): Promise<void> => {
	if (!filePath) {
		ElMessage.warning(emptyMsg);
		return;
	}

	try {
		await openExternal(filePath);
	} catch (err) {
		console.error('Failed to open file', filePath, err);
		ElMessage.error(t('workPage.messages.openFileFailed'));
	}
};

export const open3DMigotoFolder = async (): Promise<void> => {
	const base = await PathHelper.GetCurrentGame3DmigotoFolderPath();
	await openTargetPath(base, t('workPage.messages.migotoPathNotConfigured'));
};

export const openModsFolder = async (): Promise<void> => {
	const current3DmigotoFolderPath = await PathHelper.GetCurrentGame3DmigotoFolderPath();
	if (!current3DmigotoFolderPath) return;
	const p = await join(current3DmigotoFolderPath, 'Mods');
	await openTargetPath(p, t('workPage.messages.modsFolderNotFound'));
};

const resolveLatestFrameAnalysisFolderPath = async (): Promise<string | undefined> => {
	const current3DmigotoFolderPath = await PathHelper.GetCurrentGame3DmigotoFolderPath();
	if (!current3DmigotoFolderPath) return undefined;

	const entries = await readDir(current3DmigotoFolderPath);
	const folders = entries
		.filter((entry) => entry.isDirectory && !!entry.name && entry.name.startsWith('FrameAnalysis'))
		.map((entry) => entry.name as string)
		.sort((a, b) => b.localeCompare(a));

	if (folders.length === 0) return undefined;
	return join(current3DmigotoFolderPath, folders[0]);
};

export const openLatestFrameAnalysisFolder = async (
	refreshFrameAnalysisFolders: () => Promise<void>
): Promise<void> => {
	await refreshFrameAnalysisFolders();
	const latestFrameAnalysisFolderPath = await resolveLatestFrameAnalysisFolderPath();
	if (!latestFrameAnalysisFolderPath) {
		ElMessage.warning(t('workPage.messages.frameAnalysisFolderNotFound'));
		return;
	}
	await openTargetPath(latestFrameAnalysisFolderPath, t('workPage.messages.frameAnalysisFolderNotFound'));
};

export const openLatestFrameAnalysisLog = async (
	refreshFrameAnalysisFolders: () => Promise<void>
): Promise<void> => {
	await refreshFrameAnalysisFolders();
	const latestFrameAnalysisFolderPath = await resolveLatestFrameAnalysisFolderPath();
	if (!latestFrameAnalysisFolderPath) {
		ElMessage.warning(t('workPage.messages.frameAnalysisFolderNotFound'));
		return;
	}

	const p = await join(latestFrameAnalysisFolderPath, 'log.txt');
	try {
		await openTargetFile(p, t('workPage.messages.frameAnalysisLogNotFound'));
	} catch (err) {
		console.error('Failed to open FrameAnalysis log.txt', err);
		ElMessage.error(t('workPage.messages.openLogTxtFailed'));
	}
};

export const openLatestFrameAnalysisDeduped = async (
	refreshFrameAnalysisFolders: () => Promise<void>
): Promise<void> => {
	await refreshFrameAnalysisFolders();
	const latestFrameAnalysisFolderPath = await resolveLatestFrameAnalysisFolderPath();
	if (!latestFrameAnalysisFolderPath) {
		ElMessage.warning(t('workPage.messages.frameAnalysisFolderNotFound'));
		return;
	}

	const p = await join(latestFrameAnalysisFolderPath, 'deduped');
	await openTargetPath(p, t('workPage.messages.dedupedFolderNotFound'));
};

export const openSSMT4GlobalConfigsFolder = async (): Promise<void> => {
	const p = await GlobalConfig.SSMT4GlobalConfigsFolder();
	await openTargetPath(p, t('workPage.messages.unableToLocateSSMT4GlobalConfigsFolder'));
};
