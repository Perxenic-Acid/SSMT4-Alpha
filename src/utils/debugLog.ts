const DEBUG = import.meta.env.DEV

export function debugLog(prefix: string, ...args: unknown[]): void {
  if (DEBUG) {
    console.log(`[${prefix}]`, ...args)
  }
}

export function debugWarn(prefix: string, ...args: unknown[]): void {
  if (DEBUG) {
    console.warn(`[${prefix}]`, ...args)
  }
}

export function debugError(prefix: string, ...args: unknown[]): void {
  console.error(`[${prefix}]`, ...args)
}
