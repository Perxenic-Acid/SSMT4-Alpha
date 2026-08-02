import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { join } from '@tauri-apps/api/path';

export type DrawIBConfigEntry = {
	DrawIB: string;
	Alias: string;
};

export type DrawIBOption = {
	drawIB: string;
	alias: string;
};

export type DrawIBEditableRow = {
	drawIB: string;
	aliasName: string;
};

const DRAWIB_CONFIG_FILE_NAME = 'Config.json';

const normalizeEntry = (entry: unknown): DrawIBConfigEntry | undefined => {
	if (!entry || typeof entry !== 'object') {
		return undefined;
	}

	const raw = entry as { DrawIB?: unknown; Alias?: unknown };
	const drawIB = typeof raw.DrawIB === 'string' ? raw.DrawIB.trim() : '';
	const alias = typeof raw.Alias === 'string' ? raw.Alias.trim() : '';
	if (!drawIB) {
		return undefined;
	}

	return {
		DrawIB: drawIB,
		Alias: alias,
	};
};

export const parseDrawIBConfigContent = (content: string): DrawIBConfigEntry[] => {
	const parsed = JSON.parse(content);
	if (!Array.isArray(parsed)) {
		return [];
	}

	return parsed
		.map(normalizeEntry)
		.filter((entry): entry is DrawIBConfigEntry => Boolean(entry));
};

export const readDrawIBConfigFromWorkspace = async (
	workspacePath: string
): Promise<DrawIBConfigEntry[]> => {
	const configPath = await join(workspacePath, DRAWIB_CONFIG_FILE_NAME);
	const content = await readTextFile(configPath);
	return parseDrawIBConfigContent(content);
};

export const writeDrawIBConfigToWorkspace = async (
	workspacePath: string,
	entries: DrawIBConfigEntry[]
): Promise<void> => {
	const configPath = await join(workspacePath, DRAWIB_CONFIG_FILE_NAME);
	await writeTextFile(configPath, JSON.stringify(entries, null, 2));
};

export const editableRowsToDrawIBConfigEntries = (
	rows: DrawIBEditableRow[]
): DrawIBConfigEntry[] => {
	return rows
		.map(row => ({
			DrawIB: row.drawIB.trim(),
			Alias: row.aliasName.trim(),
		}))
		.filter(row => row.DrawIB.length > 0);
};

export const drawIBConfigEntriesToEditableRows = (
	entries: DrawIBConfigEntry[]
): DrawIBEditableRow[] => {
	return entries.map(entry => ({
		drawIB: entry.DrawIB,
		aliasName: entry.Alias,
	}));
};

export const drawIBConfigEntriesToOptions = (
	entries: DrawIBConfigEntry[]
): DrawIBOption[] => {
	return entries.map(entry => ({
		drawIB: entry.DrawIB,
		alias: entry.Alias,
	}));
};
