import type { UpdateInfo } from '../store/ResourceManager'

export const UPDATE_CHECK_TIMEOUT_MS = 5000

const parseVersionSegments = (version: string): number[] => {
  const normalized = (version || '').trim().replace(/^[^\d]*/, '')
  if (!normalized) {
    return []
  }

  const matches = normalized.match(/\d+/g)
  if (!matches) {
    return []
  }

  return matches.map(part => {
    const value = Number.parseInt(part, 10)
    return Number.isNaN(value) ? 0 : value
  })
}

export const isRemoteVersionNewer = (currentVersion: string, remoteVersion: string): boolean => {
  const currentSegments = parseVersionSegments(currentVersion)
  const remoteSegments = parseVersionSegments(remoteVersion)

  if (currentSegments.length === 0 || remoteSegments.length === 0) {
    return false
  }

  const maxLength = Math.max(currentSegments.length, remoteSegments.length)
  for (let index = 0; index < maxLength; index += 1) {
    const currentPart = currentSegments[index] ?? 0
    const remotePart = remoteSegments[index] ?? 0

    if (remotePart > currentPart) {
      return true
    }

    if (remotePart < currentPart) {
      return false
    }
  }

  return false
}

export const withTimeout = async <T>(promise: Promise<T>, timeoutMs: number): Promise<T | null> => {
  let timer: ReturnType<typeof setTimeout> | null = null

  try {
    return await Promise.race<T | null>([
      promise,
      new Promise<null>(resolve => {
        timer = setTimeout(() => resolve(null), timeoutMs)
      }),
    ])
  } finally {
    if (timer) {
      clearTimeout(timer)
    }
  }
}

export const pickNewerUpdate = async (
  fetchUpdate: () => Promise<UpdateInfo | null>,
  currentVersion: string,
  timeoutMs = UPDATE_CHECK_TIMEOUT_MS,
): Promise<UpdateInfo | null> => {
  try {
    const info = await withTimeout(fetchUpdate(), timeoutMs)
    if (!info) {
      return null
    }

    return isRemoteVersionNewer(currentVersion, info.version) ? info : null
  } catch (error) {
    console.warn('Silent update precheck failed:', error)
    return null
  }
}