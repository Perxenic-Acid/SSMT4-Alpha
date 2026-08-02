import { join } from '@tauri-apps/api/path';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { openPath } from '@tauri-apps/plugin-opener';
import { PathHelper } from '../../helper/PathHelper';
import { i18n } from '../../i18n';

type SkipRow = {
	skipIB: string;
	aliasName: string;
	indexCount: string;
	firstIndex: string;
};

const t = i18n.global.t;

export const generateIBSkipToMods = async (
	skipRows: SkipRow[],
	workspaceName: string
): Promise<void> => {
	const current3dmigotoFolderPath = await PathHelper.GetCurrentGame3DmigotoFolderPath();
	if (!current3dmigotoFolderPath) {
		throw new Error(t('workPage.messages.migotoFolderNotFound'));
	}

	if (!workspaceName.trim()) {
		throw new Error(t('workPage.messages.selectOrCreateWorkspaceFirst'));
	}

	const generatedFolderPath = await PathHelper.GetWorkspaceGeneratedModFolderPath(workspaceName);
	if (!generatedFolderPath) {
		throw new Error(t('workPage.messages.generatedModFolderPathResolveFailed'));
	}

	let outputContent = '';
	for (const skipibEntry of skipRows) {
		const skipIB = skipibEntry.skipIB.trim();
		if (!skipIB) continue;
		const indexCount = skipibEntry.indexCount.trim();
		const firstIndex = skipibEntry.firstIndex.trim();
		const sectionNameSuffix = [indexCount, firstIndex]
			.filter((value) => value !== '')
			.join('_');
		const sectionName = sectionNameSuffix
			? `TextureOverride_IB_${skipIB}_${sectionNameSuffix}`
			: `TextureOverride_IB_${skipIB}`;

		outputContent += `; ${skipibEntry.aliasName}\r\n`;
		outputContent += `[${sectionName}]\r\n`;
		outputContent += `hash = ${skipIB}\r\n`;
		if (firstIndex) {
			outputContent += `match_first_index = ${firstIndex}\r\n`;
		}
		if (indexCount) {
			outputContent += `match_index_count = ${indexCount}\r\n`;
		}
		outputContent += 'handling = skip\r\n';
		outputContent += '\r\n';
	}

	const outputPath = await join(generatedFolderPath, 'IBSkip.ini');
	await writeTextFile(outputPath, outputContent);
	await openPath(generatedFolderPath);
};
