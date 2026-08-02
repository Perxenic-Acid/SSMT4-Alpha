import { load } from '@tauri-apps/plugin-store'

const STORE_FILENAME = 'WindowPinStore.json'
const KEY_ALWAYS_ON_TOP = 'alwaysOnTop'

let cachedStore: Awaited<ReturnType<typeof load>> | null = null

async function getStore() {
  if (!cachedStore) {
    cachedStore = await load(STORE_FILENAME, { autoSave: true, defaults: {} })
  }
  return cachedStore
}

export async function getAlwaysOnTop(): Promise<boolean> {
  try {
    const store = await getStore()
    const val = await store.get<boolean>(KEY_ALWAYS_ON_TOP)
    return val ?? false
  } catch (error) {
    console.error('Failed to read always-on-top state from store:', error)
    return false
  }
}

export async function setAlwaysOnTop(pinned: boolean): Promise<void> {
  try {
    const store = await getStore()
    await store.set(KEY_ALWAYS_ON_TOP, pinned)
  } catch (error) {
    console.error('Failed to save always-on-top state to store:', error)
  }
}
