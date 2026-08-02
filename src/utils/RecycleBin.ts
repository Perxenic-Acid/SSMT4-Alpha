import { invoke } from '@tauri-apps/api/core'

export const moveFileToRecycleBin = async (path: string): Promise<void> => {
  await invoke('move_file_to_recycle_bin', { path })
}

export const moveDirectoryToRecycleBin = async (path: string): Promise<void> => {
  await invoke('move_dir_to_recycle_bin', { path })
}