<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { ArchivePreview, InstallFormState, InstallProgressState } from './ModsManagement.types';
import type { ModTagDefinition } from '../../store/ModTagStore';

const { t } = useI18n();

defineProps<{
  visible: boolean;
  form: InstallFormState;
  preview: ArchivePreview | null;
  progress: InstallProgressState;
  installing: boolean;
  groupOptions: string[];
  tagDefinitions: ModTagDefinition[];
}>();

const emit = defineEmits<{
  close: [];
  cancel: [];
  confirm: [];
  'update:form-targetGroup': [value: string];
}>();

const toggleTag = (tagId: string, form: InstallFormState) => {
  const idx = form.selectedTagIds.indexOf(tagId);
  if (idx === -1) {
    form.selectedTagIds = [...form.selectedTagIds, tagId];
  } else {
    form.selectedTagIds = form.selectedTagIds.filter(id => id !== tagId);
  }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="im-fade">
      <div v-if="visible" class="im-overlay" @click.self="installing ? undefined : emit('cancel')">
        <div class="im-dialog">
          <!-- Header -->
          <div class="im-header">
            <h2 class="im-title">{{ t('modsManagement.dialog.installModTitle') }}</h2>
            <button
              v-if="!installing"
              class="im-close-btn"
              @click="emit('cancel')"
              :aria-label="t('modsManagement.common.cancel')"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="im-body">
            <!-- Mod Name -->
            <div class="im-field">
              <label class="im-label">{{ t('modsManagement.fields.modName') }}</label>
              <input
                class="im-input"
                :value="form.modName"
                @input="(e: Event) => { (form as any).modName = (e.target as HTMLInputElement).value }"
                :placeholder="t('modsManagement.placeholders.modNameRecommended')"
              />
            </div>

            <!-- Group -->
            <div class="im-field">
              <label class="im-label">{{ t('modsManagement.fields.groupCharacter') }}</label>
              <input
                class="im-input"
                :value="form.targetGroup"
                @input="(e: Event) => { (form as any).targetGroup = (e.target as HTMLInputElement).value }"
                :placeholder="t('modsManagement.placeholders.groupCharacter')"
                list="im-group-datalist"
                autocomplete="off"
              />
              <datalist id="im-group-datalist">
                <option v-for="g in groupOptions" :key="g" :value="g" />
              </datalist>
            </div>

            <!-- Tags -->
            <div class="im-field" v-if="tagDefinitions.length > 0">
              <label class="im-label">{{ t('modsManagement.ui.tags') }}</label>
              <div class="im-tag-chips">
                <button
                  v-for="tag in tagDefinitions"
                  :key="tag.id"
                  type="button"
                  class="im-tag-chip"
                  :class="{ active: form.selectedTagIds.includes(tag.id) }"
                  :style="form.selectedTagIds.includes(tag.id) ? { background: tag.color + '22', borderColor: tag.color + '44', color: tag.color } : {}"
                  @click="toggleTag(tag.id, form)"
                >
                  {{ tag.name }}
                </button>
                <span v-if="tagDefinitions.length === 0" class="im-tag-none">{{ t('modsManagement.ui.noTagsYet') }}</span>
              </div>
            </div>

            <!-- File Preview -->
            <div v-if="preview" class="im-preview">
              <div class="im-preview-header">
                <span class="im-preview-title">{{ t('modsManagement.ui.filePreview') }}</span>
              </div>
              <div class="im-preview-body">
                <div class="im-preview-row">
                  <span class="im-preview-key">{{ t('modsManagement.ui.format') }}</span>
                  <span class="im-preview-val">{{ preview.format.toUpperCase() }}</span>
                </div>
                <div class="im-preview-row">
                  <span class="im-preview-key">{{ t('modsManagement.ui.fileCount') }}</span>
                  <span class="im-preview-val">{{ preview.file_count }}</span>
                </div>
                <div class="im-preview-row">
                  <span class="im-preview-key">{{ t('modsManagement.ui.rootFolders') }}</span>
                  <span class="im-preview-val">{{ preview.root_dirs.join(', ') || t('modsManagement.ui.noneFilesAtRoot') }}</span>
                </div>
                <div class="im-preview-row" :class="preview.has_ini ? 'im-valid' : 'im-invalid'">
                  <span class="im-preview-key">Status</span>
                  <span class="im-preview-val">
                    <svg v-if="preview.has_ini" class="im-status-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
                    {{ preview.has_ini ? t('modsManagement.ui.iniDetected') : t('modsManagement.ui.iniNotDetected') }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Progress -->
            <div v-if="progress.visible" class="im-progress">
              <div class="im-progress-label">{{ progress.stage }}</div>
              <div class="im-progress-bar">
                <div class="im-progress-fill" :style="{ width: progress.percent + '%' }"></div>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div class="im-footer">
            <button
              class="im-btn im-btn--cancel"
              :disabled="installing"
              @click="emit('cancel')"
            >{{ t('modsManagement.common.cancel') }}</button>
            <button
              class="im-btn im-btn--confirm"
              :disabled="installing"
              @click="emit('confirm')"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                <polyline points="17 21 17 13 7 13 7 21"/>
                <polyline points="7 3 7 8 15 8"/>
              </svg>
              {{ t('modsManagement.common.confirmInstall') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* ── Transition ── */
.im-fade-enter-active,
.im-fade-leave-active { transition: opacity 0.22s ease; }
.im-fade-enter-active .im-dialog { transition: transform 0.22s ease, opacity 0.22s ease; }
.im-fade-enter-from,
.im-fade-leave-to { opacity: 0; }
.im-fade-enter-from .im-dialog { transform: translateY(12px) scale(0.97); opacity: 0; }
.im-fade-leave-to .im-dialog { transform: translateY(8px) scale(0.98); opacity: 0; }

/* ── Overlay ── */
.im-overlay {
  position: fixed; inset: 0; z-index: 11600;
  display: flex; align-items: center; justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

/* ── Dialog ── */
.im-dialog {
  width: 480px; max-width: 92vw; max-height: 85vh;
  display: flex; flex-direction: column;
  background: var(--t-material-bg);
  border: var(--t-material-border);
  border-radius: 16px;
  box-shadow: var(--t-material-shadow);
  overflow: hidden;
}

/* ── Header ── */
.im-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 18px 22px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  flex-shrink: 0;
}

.im-title {
  margin: 0; font-size: 16px; font-weight: 700;
  color: rgba(255, 255, 255, 0.88);
  letter-spacing: 0.02em;
}

.im-close-btn {
  width: 30px; height: 30px; border: 1px solid rgba(255, 255, 255, 0.10);
  border-radius: 8px; background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.50); cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: all 0.18s ease;
}
.im-close-btn:hover {
  background: rgba(255, 70, 70, 0.12);
  border-color: rgba(255, 70, 70, 0.25);
  color: rgba(255, 140, 140, 0.85);
}

/* ── Body ── */
.im-body {
  flex: 1; overflow-y: auto; overscroll-behavior: contain;
  padding: 20px 22px;
  display: flex; flex-direction: column; gap: 16px;
}

/* ── Field ── */
.im-field { display: flex; flex-direction: column; gap: 6px; }

.im-label {
  font-size: 11px; font-weight: 700; letter-spacing: 0.6px;
  text-transform: uppercase; color: rgba(255, 255, 255, 0.45);
}

.im-input {
  width: 100%; padding: 10px 14px; border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.85); font-size: 13px;
  outline: none; box-sizing: border-box;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}
.im-input:hover { border-color: rgba(255, 255, 255, 0.16); }
.im-input:focus {
  border-color: rgba(var(--theme-surface-tint-rgb), 0.45);
  box-shadow: 0 0 0 3px rgba(var(--theme-surface-tint-rgb), 0.06);
}
.im-input::placeholder { color: rgba(255, 255, 255, 0.28); }

/* ── Tag Chips ── */
.im-tag-chips {
  display: flex; flex-wrap: wrap; gap: 6px;
  max-height: 160px;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.im-tag-chip {
  padding: 4px 10px; border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.55); font-size: 11px; font-weight: 600;
  cursor: pointer; transition: all 0.15s ease;
}
.im-tag-chip:hover {
  border-color: rgba(255, 255, 255, 0.25);
  color: rgba(255, 255, 255, 0.80);
}
.im-tag-chip.active {
  border-color: rgba(var(--theme-surface-tint-rgb), 0.40);
  color: rgba(255, 255, 255, 0.90);
}
.im-tag-none {
  font-size: 11px; color: rgba(255, 255, 255, 0.30);
}

/* ── Preview ── */
.im-preview {
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px; overflow: hidden;
  background: rgba(255, 255, 255, 0.025);
  flex-shrink: 0;
}

.im-preview-header {
  padding: 10px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(255, 255, 255, 0.03);
}

.im-preview-title {
  font-size: 12px; font-weight: 700; letter-spacing: 0.5px;
  text-transform: uppercase; color: rgba(255, 255, 255, 0.55);
}

.im-preview-body {
  padding: 12px 16px; display: flex; flex-direction: column; gap: 8px;
}

.im-preview-row {
  display: flex; justify-content: space-between; align-items: center;
  font-size: 13px; gap: 12px;
}

.im-preview-key {
  color: rgba(255, 255, 255, 0.45); flex-shrink: 0;
}

.im-preview-val {
  color: rgba(255, 255, 255, 0.72); text-align: right;
  display: flex; align-items: center; gap: 5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 260px;
}

.im-valid .im-preview-val { color: rgba(100, 230, 150, 0.85); }
.im-invalid .im-preview-val { color: rgba(230, 180, 100, 0.80); }
.im-status-icon { flex-shrink: 0; color: rgba(100, 230, 150, 0.80); }

/* ── Progress ── */
.im-progress {
  display: flex; flex-direction: column; gap: 8px;
  padding: 12px; border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(255, 255, 255, 0.03);
}

.im-progress-label {
  font-size: 12px; color: rgba(255, 255, 255, 0.60);
}

.im-progress-bar {
  height: 6px; border-radius: 3px;
  background: rgba(255, 255, 255, 0.07);
  overflow: hidden;
}

.im-progress-fill {
  height: 100%; border-radius: 3px;
  background: linear-gradient(90deg, rgba(var(--theme-surface-tint-rgb), 0.60), rgba(var(--theme-surface-tint-rgb), 0.85));
  transition: width 0.35s ease;
}

/* ── Footer ── */
.im-footer {
  display: flex; justify-content: flex-end; gap: 12px;
  padding: 14px 22px 18px;
  border-top: 1px solid rgba(255, 255, 255, 0.07);
  flex-shrink: 0;
}

/* ── Buttons ── */
.im-btn {
  height: 38px; padding: 0 20px; border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.10);
  font-size: 13px; font-weight: 600; letter-spacing: 0.02em;
  display: inline-flex; align-items: center; gap: 7px;
  cursor: pointer; transition: all 0.18s ease;
  color: rgba(255, 255, 255, 0.70);
  background: rgba(255, 255, 255, 0.05);
}
.im-btn:hover:not(:disabled) { transform: translateY(-1px); }
.im-btn:disabled { opacity: 0.35; cursor: not-allowed; }

/* Cancel — red */
.im-btn--cancel {
  background: rgba(220, 70, 70, 0.10);
  border-color: rgba(220, 70, 70, 0.18);
  color: rgba(240, 120, 120, 0.82);
}
.im-btn--cancel:hover:not(:disabled) {
  background: rgba(220, 70, 70, 0.16);
  border-color: rgba(220, 70, 70, 0.28);
  color: rgba(250, 140, 140, 0.92);
  box-shadow: 0 4px 16px rgba(220, 70, 70, 0.08);
}

/* Confirm — green */
.im-btn--confirm {
  background: rgba(70, 200, 120, 0.12);
  border-color: rgba(70, 200, 120, 0.22);
  color: rgba(120, 240, 160, 0.88);
}
.im-btn--confirm:hover:not(:disabled) {
  background: rgba(70, 200, 120, 0.18);
  border-color: rgba(70, 200, 120, 0.34);
  color: rgba(130, 250, 170, 0.95);
  box-shadow: 0 4px 16px rgba(70, 200, 120, 0.10);
}
</style>
