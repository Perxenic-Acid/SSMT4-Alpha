<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { ModManager } from '../../store/ModManager';
import type { ArchiveExportFormat, ModInfo } from './ModsManagement.types';

const { t } = useI18n();

const props = defineProps<{
  visible: boolean;
  mod: ModInfo | null;
}>();

const emit = defineEmits<{
  close: [];
  'refresh-after': [];
}>();

const archiveExportFormats: ArchiveExportFormat[] = ['zip', '7z', 'rar'];

const exporting = ref(false);
const archiveName = ref('');
const outputDir = ref('');
const password = ref('');
const format = ref<ArchiveExportFormat>('zip');

const openDialog = (mod: ModInfo) => {
  exporting.value = false;
  archiveName.value = mod.name.replace(/[\\/:*?"<>|]/g, '_').trim() || 'Mod';
  outputDir.value = '';
  password.value = '';
  format.value = 'zip';
};

defineExpose({ openDialog });

const isValidArchiveFileName = (value: string) => {
  const trimmed = value.trim();
  return !!trimmed && /^[^\\/:*?"<>|]+$/.test(trimmed) && !trimmed.endsWith('.') && !trimmed.endsWith(' ');
};

const chooseOutputDir = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('modsManagement.dialog.selectExportFolderTitle') });
    if (selected && typeof selected === 'string') { outputDir.value = selected; return selected; }
  } catch (error) { ElMessage.error(t('modsManagement.messages.selectExportFolderFailed', { error: String(error) })); }
  return '';
};

const confirmExport = async () => {
  if (exporting.value || !props.mod) return;
  if (!isValidArchiveFileName(archiveName.value)) { ElMessage.warning(t('modsManagement.messages.nameContainsInvalidCharacters')); return; }
  let dir = outputDir.value.trim();
  if (!dir) { dir = await chooseOutputDir(); if (!dir) return; }
  exporting.value = true;
  try {
    const installDir = await ModManager.getInstallDir('');
    const outputPath = await invoke<string>('export_mod_archive', { installDir, modRelativePath: props.mod.relativePath, outputDir: dir, archiveName: archiveName.value.trim(), format: format.value, password: password.value.trim() || null });
    ElMessage.success(t('modsManagement.messages.exportArchiveSuccess', { path: outputPath }));
    emit('close');
    try { await revealItemInDir(outputPath); } catch { await openPath(dir); }
  } catch (error) {
    ElMessage.error(t('modsManagement.messages.exportArchiveFailed', { error: String(error) }));
    if (String(error).includes('Mod list may be stale')) emit('refresh-after');
  } finally { exporting.value = false; }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="ex-fade">
      <div v-if="visible && mod" class="ex-overlay" @click.self="exporting ? undefined : emit('close')">
        <div class="ex-dialog">
          <div class="ex-header">
            <h2 class="ex-title">{{ t('modsManagement.dialog.exportArchiveTitle') }}</h2>
            <button v-if="!exporting" class="ex-close-btn" @click="emit('close')" :aria-label="t('modsManagement.common.cancel')">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="ex-body">
            <div class="ex-field">
              <label class="ex-label">{{ t('modsManagement.fields.modName') }}</label>
              <input class="ex-input" :value="mod.name" disabled />
            </div>
            <div class="ex-field">
              <label class="ex-label">{{ t('modsManagement.fields.archiveName') }}</label>
              <input class="ex-input" v-model="archiveName" :placeholder="t('modsManagement.placeholders.archiveName')" :disabled="exporting" />
            </div>
            <div class="ex-field">
              <label class="ex-label">{{ t('modsManagement.fields.outputFolder') }}</label>
              <div class="ex-row">
                <input class="ex-input ex-input--flex" :value="outputDir" :placeholder="t('modsManagement.placeholders.outputFolder')" readonly :disabled="exporting" />
                <button class="ex-btn ex-btn--outline" @click="chooseOutputDir" :disabled="exporting">{{ t('modsManagement.actions.chooseFolder') }}</button>
              </div>
            </div>
            <div class="ex-field">
              <label class="ex-label">{{ t('modsManagement.fields.archiveFormat') }}</label>
              <select class="ex-input" v-model="format" :disabled="exporting">
                <option v-for="f in archiveExportFormats" :key="f" :value="f">{{ f.toUpperCase() }}</option>
              </select>
            </div>
          </div>
          <div class="ex-footer">
            <button class="ex-btn ex-btn--cancel" :disabled="exporting" @click="emit('close')">{{ t('modsManagement.common.cancel') }}</button>
            <button class="ex-btn ex-btn--confirm" :disabled="exporting" @click="confirmExport">{{ exporting ? '...' : t('modsManagement.common.confirm') }}</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.ex-fade-enter-active, .ex-fade-leave-active { transition: opacity .22s ease; }
.ex-fade-enter-from, .ex-fade-leave-to { opacity: 0; }
.ex-fade-enter-active .ex-dialog { transition: transform .22s ease, opacity .22s ease; }
.ex-fade-enter-from .ex-dialog { transform: translateY(12px) scale(.97); opacity: 0; }

.ex-overlay { position:fixed; inset:0; z-index:11610; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,.45); backdrop-filter:blur(4px); -webkit-backdrop-filter:blur(4px); }
.ex-dialog { width:460px; max-width:92vw; display:flex; flex-direction:column; background:var(--t-material-bg); border:var(--t-material-border); border-radius:16px; box-shadow:var(--t-material-shadow); overflow:hidden; }
.ex-header { display:flex; align-items:center; justify-content:space-between; padding:18px 22px 14px; border-bottom:1px solid rgba(255,255,255,.07); }
.ex-title { margin:0; font-size:16px; font-weight:700; color:rgba(255,255,255,.88); }
.ex-close-btn { width:30px; height:30px; border:1px solid rgba(255,255,255,.10); border-radius:8px; background:rgba(255,255,255,.04); color:rgba(255,255,255,.50); cursor:pointer; display:flex; align-items:center; justify-content:center; transition:all .18s; }
.ex-close-btn:hover { background:rgba(255,70,70,.12); border-color:rgba(255,70,70,.25); color:rgba(255,140,140,.85); }
.ex-body { flex:1; overflow-y:auto; padding:20px 22px; display:flex; flex-direction:column; gap:14px; }
.ex-field { display:flex; flex-direction:column; gap:6px; }
.ex-label { font-size:11px; font-weight:700; letter-spacing:.6px; text-transform:uppercase; color:rgba(255,255,255,.45); }
.ex-input { width:100%; padding:10px 14px; border-radius:10px; border:1px solid rgba(255,255,255,.09); background:rgba(255,255,255,.04); color:rgba(255,255,255,.85); font-size:13px; outline:none; box-sizing:border-box; transition:border-color .2s; }
.ex-input:hover { border-color:rgba(255,255,255,.16); }
.ex-input:focus { border-color:rgba(var(--theme-surface-tint-rgb),.45); box-shadow:0 0 0 3px rgba(var(--theme-surface-tint-rgb),.06); }
.ex-input:disabled { opacity:.45; }
.ex-input--flex { flex:1; min-width:0; }
.ex-row { display:flex; gap:8px; }
.ex-footer { display:flex; justify-content:flex-end; gap:12px; padding:14px 22px 18px; border-top:1px solid rgba(255,255,255,.07); }
.ex-btn { height:38px; padding:0 20px; border-radius:10px; border:1px solid rgba(255,255,255,.10); font-size:13px; font-weight:600; cursor:pointer; transition:all .18s; color:rgba(255,255,255,.70); background:rgba(255,255,255,.05); }
.ex-btn:hover:not(:disabled) { transform:translateY(-1px); }
.ex-btn:disabled { opacity:.35; cursor:not-allowed; }
.ex-btn--outline { height:auto; padding:10px 14px; white-space:nowrap; }
.ex-btn--cancel { background:rgba(220,70,70,.10); border-color:rgba(220,70,70,.18); color:rgba(240,120,120,.82); }
.ex-btn--cancel:hover:not(:disabled) { background:rgba(220,70,70,.16); border-color:rgba(220,70,70,.28); color:rgba(250,140,140,.92); }
.ex-btn--confirm { background:rgba(70,200,120,.12); border-color:rgba(70,200,120,.22); color:rgba(120,240,160,.88); }
.ex-btn--confirm:hover:not(:disabled) { background:rgba(70,200,120,.18); border-color:rgba(70,200,120,.34); color:rgba(130,250,170,.95); }
select.ex-input { appearance:none; -webkit-appearance:none; cursor:pointer; background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='rgba(255,255,255,0.4)' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'/%3E%3C/svg%3E"); background-repeat:no-repeat; background-position:right 14px center; padding-right:36px; }
</style>
