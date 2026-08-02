import { exists, mkdir, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { defineStore } from 'pinia'
import { PathHelper } from '../helper/PathHelper'

export interface RememberedModVariableState {
  runtimeKey: string
  variableName: string
  value: string
}

export interface RememberedModState {
  modId: string
  updatedAt: number
  variables: RememberedModVariableState[]
}

export interface ModStateSnapshot {
  version: number
  modStates: Record<string, RememberedModState>
}

const MOD_STATE_VERSION = 1
const MOD_STATE_DIR_NAME = '.mod-state'
const MOD_STATE_FILE_NAME = 'mod-state.json'

const normalizePath = (value: string) => value.replace(/\\/g, '/').replace(/\/+$/g, '').replace(/\/+/g, '/')

const joinPath = (...parts: string[]) => parts
  .map((part, index) => {
    const normalized = normalizePath(part)
    if (index === 0) return normalized.replace(/\/+$/g, '')
    return normalized.replace(/^\/+|\/+$/g, '')
  })
  .filter(Boolean)
  .join('/')

const stripDisabledPrefix = (segment: string) => {
  const upper = segment.toUpperCase()
  if (upper.startsWith('DISABLED_')) return segment.slice(9)
  if (upper.startsWith('DISABLED')) return segment.slice(8)
  return segment
}

export const normalizeModStateId = (modId: string) => normalizePath(modId)
  .split('/')
  .filter(Boolean)
  .map(stripDisabledPrefix)
  .join('/')

const normalizeVariableState = (value: unknown): RememberedModVariableState | null => {
  if (!value || typeof value !== 'object') return null
  const candidate = value as Partial<RememberedModVariableState>
  const runtimeKey = typeof candidate.runtimeKey === 'string' ? candidate.runtimeKey.trim() : ''
  const variableName = typeof candidate.variableName === 'string' ? candidate.variableName.trim() : ''
  const rawValue = typeof candidate.value === 'string' ? candidate.value : ''
  if (!runtimeKey || !variableName) return null
  return { runtimeKey, variableName, value: rawValue }
}

const normalizeModState = (value: unknown): RememberedModState | null => {
  if (!value || typeof value !== 'object') return null
  const candidate = value as Partial<RememberedModState>
  const modId = typeof candidate.modId === 'string' ? normalizeModStateId(candidate.modId) : ''
  if (!modId) return null
  const updatedAt = Number.isFinite(candidate.updatedAt) ? Number(candidate.updatedAt) : Date.now()
  const variables = Array.isArray(candidate.variables)
    ? candidate.variables.map(normalizeVariableState).filter((entry): entry is RememberedModVariableState => entry !== null)
    : []
  return { modId, updatedAt, variables }
}

export const useModStateStore = defineStore('modState', () => {
  async function getModsRoot(gameName: string) {
    const installDir = await PathHelper.GetGame3DmigotoFolderPath(gameName)
    if (!installDir) {
      throw new Error('3Dmigoto install directory is not configured')
    }
    return joinPath(installDir, 'Mods')
  }

  async function getStoreDir(gameName: string) {
    const modsRoot = await getModsRoot(gameName)
    return joinPath(modsRoot, MOD_STATE_DIR_NAME)
  }

  async function getStoreFile(gameName: string) {
    const storeDir = await getStoreDir(gameName)
    return joinPath(storeDir, MOD_STATE_FILE_NAME)
  }

  async function ensureStoreDir(gameName: string) {
    const storeDir = await getStoreDir(gameName)
    if (!(await exists(storeDir))) {
      await mkdir(storeDir, { recursive: true })
    }
  }

  async function writeSnapshot(gameName: string, snapshot: ModStateSnapshot) {
    await ensureStoreDir(gameName)
    const path = await getStoreFile(gameName)
    await writeTextFile(path, JSON.stringify(snapshot, null, 2))
    return snapshot
  }

  async function load(gameName: string): Promise<ModStateSnapshot> {
    const path = await getStoreFile(gameName)
    if (!(await exists(path))) {
      return { version: MOD_STATE_VERSION, modStates: {} }
    }

    try {
      const raw = await readTextFile(path)
      const parsed = JSON.parse(raw) as Partial<ModStateSnapshot>
      const next: Record<string, RememberedModState> = {}
      Object.entries(parsed.modStates || {}).forEach(([modId, state]) => {
        const normalized = normalizeModState(state)
        if (!normalized || normalized.variables.length === 0) return
        next[normalizeModStateId(modId)] = normalized
      })
      return { version: MOD_STATE_VERSION, modStates: next }
    } catch {
      return { version: MOD_STATE_VERSION, modStates: {} }
    }
  }

  async function saveModState(gameName: string, modId: string, variables: RememberedModVariableState[]) {
    const normalizedModId = normalizeModStateId(modId)
    const snapshot = await load(gameName)
    const cleanVariables = variables
      .map(normalizeVariableState)
      .filter((entry): entry is RememberedModVariableState => entry !== null)

    const nextStates = { ...snapshot.modStates }
    if (cleanVariables.length === 0) {
      delete nextStates[normalizedModId]
    } else {
      nextStates[normalizedModId] = {
        modId: normalizedModId,
        updatedAt: Date.now(),
        variables: cleanVariables,
      }
    }

    await writeSnapshot(gameName, {
      version: MOD_STATE_VERSION,
      modStates: nextStates,
    })
    return nextStates[normalizedModId] || null
  }

  async function getModState(gameName: string, modId: string) {
    const snapshot = await load(gameName)
    return snapshot.modStates[normalizeModStateId(modId)] || null
  }

  async function remapModPath(gameName: string, oldModId: string, newModId: string) {
    const oldKey = normalizeModStateId(oldModId)
    const newKey = normalizeModStateId(newModId)
    if (!oldKey || !newKey || oldKey === newKey) {
      return load(gameName)
    }

    const snapshot = await load(gameName)
    const state = snapshot.modStates[oldKey]
    if (!state) return snapshot

    const nextStates = { ...snapshot.modStates }
    delete nextStates[oldKey]
    nextStates[newKey] = {
      ...state,
      modId: newKey,
    }

    return writeSnapshot(gameName, {
      version: MOD_STATE_VERSION,
      modStates: nextStates,
    })
  }

  async function remapPrefix(gameName: string, oldPrefix: string, newPrefix: string) {
    const cleanOldPrefix = normalizeModStateId(oldPrefix)
    const cleanNewPrefix = normalizeModStateId(newPrefix)
    if (!cleanOldPrefix || !cleanNewPrefix || cleanOldPrefix === cleanNewPrefix) {
      return load(gameName)
    }

    const snapshot = await load(gameName)
    const nextStates: Record<string, RememberedModState> = {}

    Object.entries(snapshot.modStates).forEach(([modId, state]) => {
      const cleanModId = normalizeModStateId(modId)
      if (cleanModId === cleanOldPrefix || cleanModId.startsWith(`${cleanOldPrefix}/`)) {
        const suffix = cleanModId.slice(cleanOldPrefix.length)
        const nextKey = `${cleanNewPrefix}${suffix}`
        nextStates[nextKey] = {
          ...state,
          modId: nextKey,
        }
      } else {
        nextStates[cleanModId] = state
      }
    })

    return writeSnapshot(gameName, {
      version: MOD_STATE_VERSION,
      modStates: nextStates,
    })
  }

  return { load, saveModState, getModState, remapModPath, remapPrefix }
})

/**
 * Backward-compatible wrapper that delegates to the Pinia store.
 * Existing callers using `ModStateStore.saveModState(...)` etc. continue to work.
 * @deprecated Prefer `useModStateStore()` in new code.
 */
export const ModStateStore = {
  get load() { return useModStateStore().load },
  get saveModState() { return useModStateStore().saveModState },
  get getModState() { return useModStateStore().getModState },
  get remapModPath() { return useModStateStore().remapModPath },
  get remapPrefix() { return useModStateStore().remapPrefix },
}
