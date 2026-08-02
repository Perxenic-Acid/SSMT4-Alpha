import { join } from '@tauri-apps/api/path'
import { exists } from '@tauri-apps/plugin-fs'
import type { ModInfo, ModKeyInfo } from '../../store/ModManager'
import { D3dxIniManager } from '../../store/D3dxIniManager'
import { ModStateStore } from '../../store/ModStateStore'
import { PathHelper } from '../../helper/PathHelper'
import {
  buildModRuntimeRestoreValues,
  collectModRuntimeVariables,
  removeModRuntimeEntriesFromLines,
} from './D3dxUserIniVariables'

const resolveUserIniPath = async (gameName: string) => {
  const migotoDir = await PathHelper.GetGame3DmigotoFolderPath(gameName)
  if (!migotoDir) {
    throw new Error('3Dmigoto install directory is not configured')
  }

  return join(migotoDir, 'd3dx_user.ini')
}

export const captureModRuntimeState = async (gameName: string, mod: Pick<ModInfo, 'relativePath'>, _items: ModKeyInfo[]) => {
  const userIniPath = await resolveUserIniPath(gameName)
  if (!(await exists(userIniPath))) {
    return { saved: false, reason: 'missing-user-ini' as const }
  }

  const lines = await D3dxIniManager.loadIni(userIniPath)
  const constantsEntries = D3dxIniManager.getSectionEntries(lines, 'Constants')
  const variables = collectModRuntimeVariables(mod.relativePath, constantsEntries)
  if (variables.length === 0) {
    await ModStateStore.saveModState(gameName, mod.relativePath, [])
    return { saved: false, reason: 'no-matched-variables' as const }
  }

  await ModStateStore.saveModState(gameName, mod.relativePath, variables)
  return { saved: true, count: variables.length }
}

export const restoreModRuntimeState = async (gameName: string, mod: Pick<ModInfo, 'relativePath'>) => {
  const remembered = await ModStateStore.getModState(gameName, mod.relativePath)
  if (!remembered || remembered.variables.length === 0) {
    return { restored: false, reason: 'missing-state' as const }
  }

  const userIniPath = await resolveUserIniPath(gameName)
  if (!(await exists(userIniPath))) {
    return { restored: false, reason: 'missing-user-ini' as const }
  }

  let lines = await D3dxIniManager.loadIni(userIniPath)
  const constantsEntries = D3dxIniManager.getSectionEntries(lines, 'Constants')
  lines = removeModRuntimeEntriesFromLines(lines, mod.relativePath, constantsEntries)

  const runtimeValues = buildModRuntimeRestoreValues(mod.relativePath, remembered.variables)
  lines = D3dxIniManager.setIniValues(lines, 'Constants', runtimeValues)
  await D3dxIniManager.saveIni(userIniPath, lines)
  return { restored: true, count: Object.keys(runtimeValues).length }
}
