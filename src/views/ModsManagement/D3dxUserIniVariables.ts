import type { IniSectionEntry } from '../../store/D3dxIniManager'
import type { RememberedModVariableState } from '../../store/ModStateStore'

type RuntimeEntry = Pick<IniSectionEntry, 'key' | 'value'>
type RuntimeEntryWithIndex = Pick<IniSectionEntry, 'key' | 'lineIndex'>

const stripDisabledPrefix = (segment: string) => {
  const upper = segment.toUpperCase()
  if (upper.startsWith('DISABLED_')) return segment.slice(9)
  if (upper.startsWith('DISABLED')) return segment.slice(8)
  return segment
}

const normalizeModPath = (value: string) => String(value || '')
  .replace(/\\/g, '/')
  .replace(/\/+/g, '/')
  .replace(/^\/+|\/+$/g, '')
  .split('/')
  .filter(Boolean)
  .map(stripDisabledPrefix)
  .join('/')

const normalizeRuntimeSeparators = (value: string) => String(value || '')
  .trim()
  .replace(/\//g, '\\')
  .replace(/\\+/g, '\\')

const normalizeRuntimeKeyForCompare = (value: string) => normalizeRuntimeSeparators(value).toLowerCase()

export const buildModRuntimePrefix = (modRelativePath: string) => {
  const normalizedPath = normalizeModPath(modRelativePath)
  if (!normalizedPath) return ''
  return `$\\mods\\${normalizedPath.replace(/\//g, '\\')}\\`.toLowerCase()
}

const getRuntimeSuffixForMod = (runtimeKey: string, modRelativePath: string) => {
  const runtimePrefix = buildModRuntimePrefix(modRelativePath)
  if (!runtimePrefix) return ''

  const normalizedKey = normalizeRuntimeSeparators(runtimeKey)
  const keyForCompare = normalizeRuntimeKeyForCompare(normalizedKey)
  const prefixForCompare = normalizeRuntimeKeyForCompare(runtimePrefix)
  if (!keyForCompare.startsWith(prefixForCompare)) return ''

  return normalizedKey.slice(prefixForCompare.length).trim()
}

const normalizeFallbackVariableSuffix = (variableName: string) => normalizeRuntimeSeparators(variableName)
  .replace(/^\$/, '')
  .replace(/^\\+/, '')

export const collectModRuntimeVariables = (
  modRelativePath: string,
  constantsEntries: RuntimeEntry[],
): RememberedModVariableState[] => {
  const variablesByRuntimeKey = new Map<string, RememberedModVariableState>()

  constantsEntries.forEach((entry) => {
    const runtimeKey = normalizeRuntimeSeparators(entry.key)
    const variableName = getRuntimeSuffixForMod(runtimeKey, modRelativePath)
    if (!runtimeKey || !variableName) return

    variablesByRuntimeKey.set(normalizeRuntimeKeyForCompare(runtimeKey), {
      runtimeKey,
      variableName,
      value: entry.value,
    })
  })

  return Array.from(variablesByRuntimeKey.values())
}

export const getModRuntimeLineIndexes = (
  modRelativePath: string,
  constantsEntries: RuntimeEntryWithIndex[],
) => constantsEntries
  .filter((entry) => getRuntimeSuffixForMod(entry.key, modRelativePath))
  .map((entry) => entry.lineIndex)

export const removeModRuntimeEntriesFromLines = (
  lines: string[],
  modRelativePath: string,
  constantsEntries: RuntimeEntryWithIndex[],
) => {
  const nextLines = [...lines]
  getModRuntimeLineIndexes(modRelativePath, constantsEntries)
    .sort((left, right) => right - left)
    .forEach((lineIndex) => {
      nextLines.splice(lineIndex, 1)
    })
  return nextLines
}

export const buildModRuntimeRestoreValues = (
  modRelativePath: string,
  variables: RememberedModVariableState[],
) => {
  const runtimePrefix = buildModRuntimePrefix(modRelativePath)
  if (!runtimePrefix) return {}

  return Object.fromEntries(variables
    .map((entry) => {
      const suffix = getRuntimeSuffixForMod(entry.runtimeKey, modRelativePath)
        || normalizeFallbackVariableSuffix(entry.variableName)
      const runtimeKey = suffix ? `${runtimePrefix}${suffix}` : ''
      return [runtimeKey, entry.value]
    })
    .filter(([runtimeKey]) => runtimeKey))
}
