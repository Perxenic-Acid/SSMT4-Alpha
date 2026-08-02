import { readTextFile } from '@tauri-apps/plugin-fs'

export class SSMTJsonUtils {
	// Read JSON file; return parsed object or null if missing/invalid.
	// Returns unknown — callers must validate structure or use readJsonOrDefault.
	public static async readJson(path: string): Promise<unknown | null> {
		try {
			const raw = await readTextFile(path)
			return JSON.parse(raw) as unknown
		} catch {
			return null
		}
	}

	// Read JSON file; fall back to provided default value on any error.
	// NOTE: Uses unchecked cast to T — callers are responsible for ensuring
	// the JSON file structure matches the expected type at runtime.
	public static async readJsonOrDefault<T>(path: string, fallback: T): Promise<T> {
		const parsed = await SSMTJsonUtils.readJson(path)
		return parsed == null ? fallback : parsed as T
	}
}