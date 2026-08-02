export type TextureMarkNameMemoryItem = {
	name: string
	createdAt: number
	updatedAt: number
	usedCount: number
}

export type TextureMarkNameMemoryConfig = {
	version: 1
	byPreset: Record<string, TextureMarkNameMemoryItem[]>
}

const STORAGE_KEY = 'ssmt4_mark_texture_custom_mark_names_v1'
const MAX_NAMES_PER_PRESET = 100

const normalizePreset = (preset: string): string => {
	const normalized = (preset || '').trim().toUpperCase()
	return normalized || 'DEFAULT'
}

const normalizeNameKey = (name: string): string => name.trim().toLocaleLowerCase()

const createEmptyConfig = (): TextureMarkNameMemoryConfig => ({
	version: 1,
	byPreset: {},
})

const isStorageAvailable = (): boolean => typeof localStorage !== 'undefined'

const sanitizeItemList = (items: unknown): TextureMarkNameMemoryItem[] => {
	if (!Array.isArray(items)) {
		return []
	}

	const seen = new Set<string>()
	const sanitized: TextureMarkNameMemoryItem[] = []

	for (const item of items) {
		if (!item || typeof item !== 'object') {
			continue
		}

		const source = item as Partial<TextureMarkNameMemoryItem>
		const name = typeof source.name === 'string' ? source.name.trim() : ''
		const key = normalizeNameKey(name)
		if (!name || seen.has(key)) {
			continue
		}

		const createdAt = typeof source.createdAt === 'number' ? source.createdAt : Date.now()
		const updatedAt = typeof source.updatedAt === 'number' ? source.updatedAt : createdAt
		const usedCount = typeof source.usedCount === 'number' ? source.usedCount : 1

		seen.add(key)
		sanitized.push({
			name,
			createdAt,
			updatedAt,
			usedCount: Math.max(0, Math.floor(usedCount)),
		})
	}

	return sanitized
		.sort((a, b) => b.updatedAt - a.updatedAt || b.usedCount - a.usedCount || a.name.localeCompare(b.name))
		.slice(0, MAX_NAMES_PER_PRESET)
}

const loadConfig = (): TextureMarkNameMemoryConfig => {
	if (!isStorageAvailable()) {
		return createEmptyConfig()
	}

	try {
		const raw = localStorage.getItem(STORAGE_KEY)
		if (!raw) {
			return createEmptyConfig()
		}

		const parsed = JSON.parse(raw) as Partial<TextureMarkNameMemoryConfig>
		const config = createEmptyConfig()
		if (parsed && typeof parsed.byPreset === 'object' && parsed.byPreset) {
			for (const [preset, items] of Object.entries(parsed.byPreset)) {
				const presetKey = normalizePreset(preset)
				const list = sanitizeItemList(items)
				if (list.length > 0) {
					config.byPreset[presetKey] = list
				}
			}
		}

		return config
	} catch {
		return createEmptyConfig()
	}
}

const saveConfig = (config: TextureMarkNameMemoryConfig): void => {
	if (!isStorageAvailable()) {
		return
	}

	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(config))
	} catch {
		// Ignore quota or privacy-mode failures; marking itself should keep working.
	}
}

export const getCustomTextureMarkNames = (preset: string): TextureMarkNameMemoryItem[] => {
	const config = loadConfig()
	return [...(config.byPreset[normalizePreset(preset)] || [])]
}

export const addCustomTextureMarkName = (
	preset: string,
	name: string
): TextureMarkNameMemoryItem | null => {
	const normalizedName = name.trim()
	if (!normalizedName) {
		return null
	}

	const config = loadConfig()
	const presetKey = normalizePreset(preset)
	const list = [...(config.byPreset[presetKey] || [])]
	const key = normalizeNameKey(normalizedName)
	const existingIndex = list.findIndex(item => normalizeNameKey(item.name) === key)
	const now = Date.now()

	if (existingIndex >= 0) {
		const existing = list[existingIndex]
		const updated = {
			...existing,
			updatedAt: now,
			usedCount: existing.usedCount + 1,
		}
		list.splice(existingIndex, 1, updated)
		config.byPreset[presetKey] = sanitizeItemList(list)
		saveConfig(config)
		return updated
	}

	const created: TextureMarkNameMemoryItem = {
		name: normalizedName,
		createdAt: now,
		updatedAt: now,
		usedCount: 1,
	}

	list.unshift(created)
	config.byPreset[presetKey] = sanitizeItemList(list)
	saveConfig(config)
	return created
}

export const removeCustomTextureMarkName = (preset: string, name: string): boolean => {
	const config = loadConfig()
	const presetKey = normalizePreset(preset)
	const list = config.byPreset[presetKey] || []
	const key = normalizeNameKey(name)
	const nextList = list.filter(item => normalizeNameKey(item.name) !== key)

	if (nextList.length === list.length) {
		return false
	}

	if (nextList.length > 0) {
		config.byPreset[presetKey] = nextList
	} else {
		delete config.byPreset[presetKey]
	}

	saveConfig(config)
	return true
}
