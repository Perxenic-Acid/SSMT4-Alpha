import { AppStateManager, BGType, SSMTLocale, type GameInfo } from './store/AppStateManager'

/**
 * Legacy compatibility layer: re-export AppStateManager for callers that still import from ./store.
 */
export const useLegacyAppStateStore = () => AppStateManager
export { BGType, SSMTLocale }
export type { GameInfo }