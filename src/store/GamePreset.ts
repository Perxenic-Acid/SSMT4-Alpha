export const GAME_PRESET_VALUES = [
  'GIMI',
  'HIMI',
  'SRMI',
  'ZZMI',
  'ZZMIDX12',
  'WWMI',
  'EFMI',
  'NTEMI',
  'GF2',
  'IdentityV',
  'AILIMIT',
  'DOAV',
  'SnowBreak',
  'YYSLS',
  'APMI',
  'Naraka',
  'NarakaM',
] as const

export type GamePreset = (typeof GAME_PRESET_VALUES)[number]

const GAME_PRESET_SET: ReadonlySet<string> = new Set(GAME_PRESET_VALUES)

export const MIHOYO_GAME_PRESET_VALUES = [
  'GIMI',
  'HIMI',
  'SRMI',
  'ZZMI',
  'ZZMIDX12',
] as const

const MIHOYO_GAME_PRESET_SET: ReadonlySet<string> = new Set(MIHOYO_GAME_PRESET_VALUES)

export const GAME_PRESET_OPTIONS: ReadonlyArray<{ label: GamePreset; value: GamePreset }> =
  GAME_PRESET_VALUES.map(preset => ({
    label: preset,
    value: preset,
  }))

export function isMihoyoGamePreset(gamePreset: unknown): boolean {
  const normalizedPreset = typeof gamePreset === 'string' ? gamePreset.trim().toUpperCase() : ''
  return MIHOYO_GAME_PRESET_SET.has(normalizedPreset)
}

export const AUTO_UPDATE_SUPPORTED_PRESETS = [
  'GIMI',
  'HIMI',
  'SRMI',
  'ZZMI',
  'ZZMIDX12',
  'NTEMI',
  'WWMI',
] as const

export const AUTO_UPDATE_SUPPORTED_PRESET_SET: ReadonlySet<string> =
  new Set(AUTO_UPDATE_SUPPORTED_PRESETS)

export const GAME_PRESET_GITHUB_REPO_MAP: Readonly<Record<GamePreset, string>> = {
  GIMI: 'SilentNightSound/GIMI-Package',
  HIMI: 'leotorrez/HIMI-Package',
  SRMI: 'SpectrumQT/SRMI-Package',
  ZZMI: 'leotorrez/ZZMI-Package',
  ZZMIDX12: 'StarBobis/MinBase-Package',
  WWMI: 'SpectrumQT/WWMI-Package',
  EFMI: 'SpectrumQT/EFMI-Package',
  
  //异环暂时用222221的包
  NTEMI: 'ssice-a/NTMI-PACKAGE',
  
  GF2: 'SilentNightSound/GIMI-Package',
  IdentityV: 'StarBobis/MinBase-Package',
  AILIMIT: 'StarBobis/MinBase-Package',
  DOAV: 'StarBobis/MinBase-Package',
  SnowBreak: 'StarBobis/MinBase-Package',
  YYSLS: 'StarBobis/MinBase-Package',
  APMI: 'StarBobis/MinBase-Package',
  Naraka: 'StarBobis/NBP-Package',
  NarakaM: 'StarBobis/NBPM-Package',
}

export function getGithubRepoByGamePreset(gamePreset: string): string | null {
  if (Object.prototype.hasOwnProperty.call(GAME_PRESET_GITHUB_REPO_MAP, gamePreset)) {
    return GAME_PRESET_GITHUB_REPO_MAP[gamePreset as GamePreset]
  }
  return null
}

export function resolveGamePresetByGameName(gameName: string): GamePreset | null {
  if (GAME_PRESET_SET.has(gameName)) {
    return gameName as GamePreset
  }
  return null
}

export function isValidGamePreset(gamePreset: string | null | undefined): boolean {
  const normalizedPreset = (gamePreset || '').trim()
  if (!normalizedPreset) {
    return false
  }
  return GAME_PRESET_SET.has(normalizedPreset)
}
