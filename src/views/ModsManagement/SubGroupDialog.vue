<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { ModManager } from '../../store/ModManager';

const { t } = useI18n();

const props = defineProps<{
  visible: boolean;
  parentId: string;
}>();

const emit = defineEmits<{
  close: [];
  created: [];
}>();

const name = ref('');
const icon = ref('');

const reset = () => { name.value = ''; icon.value = ''; };
defineExpose({ reset });

const pickIcon = async () => {
  const picked = await open({ multiple: false, filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp'] }] });
  if (picked) icon.value = picked;
};

const confirm = async () => {
  if (!name.value) { ElMessage.warning(t('modsManagement.messages.enterSubcategoryName')); return; }
  const newGroupPath = props.parentId ? `${props.parentId}/${name.value}` : name.value;
  try {
    await ModManager.createModGroup('', newGroupPath);
    if (icon.value) {
      try { await ModManager.setModGroupIcon('', newGroupPath, icon.value); }
      catch (e: unknown) { ElMessage.warning(t('modsManagement.messages.subcategoryIconSetFailed', { error: String(e) })); }
    }
    ElMessage.success(t('modsManagement.messages.subcategoryCreated'));
    emit('created');
  } catch (e: unknown) { ElMessage.error(t('modsManagement.messages.createFailed', { error: String(e) })); }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="sg-fade">
      <div v-if="visible" class="sg-overlay" @click.self="emit('close')">
        <div class="sg-dialog">
          <div class="sg-header">
            <h2 class="sg-title">{{ t('modsManagement.dialog.createSubcategoryTitle') }}</h2>
            <button class="sg-close-btn" @click="emit('close')">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="sg-body">
            <div class="sg-field">
              <label class="sg-label">{{ t('modsManagement.fields.name') }}</label>
              <input class="sg-input" v-model="name" :placeholder="t('modsManagement.placeholders.subcategoryName')" />
            </div>
            <div class="sg-field">
              <label class="sg-label">{{ t('modsManagement.fields.iconOptional') }}</label>
              <div class="sg-row">
                <input class="sg-input sg-input--flex" :value="icon" :placeholder="t('modsManagement.ui.notSelected')" readonly />
                <button class="sg-btn sg-btn--outline" @click="pickIcon">{{ t('modsManagement.actions.chooseIcon') }}</button>
              </div>
            </div>
          </div>
          <div class="sg-footer">
            <button class="sg-btn sg-btn--cancel" @click="emit('close')">{{ t('modsManagement.common.cancel') }}</button>
            <button class="sg-btn sg-btn--confirm" @click="confirm">{{ t('modsManagement.common.confirm') }}</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.sg-fade-enter-active,.sg-fade-leave-active{transition:opacity .22s ease}
.sg-fade-enter-from,.sg-fade-leave-to{opacity:0}
.sg-overlay{position:fixed;inset:0;z-index:11610;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.45);backdrop-filter:blur(4px)}
.sg-dialog{width:420px;max-width:92vw;display:flex;flex-direction:column;background:var(--t-material-bg);border:var(--t-material-border);border-radius:16px;box-shadow:var(--t-material-shadow);overflow:hidden}
.sg-header{display:flex;align-items:center;justify-content:space-between;padding:18px 22px 14px;border-bottom:1px solid rgba(255,255,255,.07)}
.sg-title{margin:0;font-size:16px;font-weight:700;color:rgba(255,255,255,.88)}
.sg-close-btn{width:30px;height:30px;border:1px solid rgba(255,255,255,.10);border-radius:8px;background:rgba(255,255,255,.04);color:rgba(255,255,255,.50);cursor:pointer;display:flex;align-items:center;justify-content:center}
.sg-close-btn:hover{background:rgba(255,70,70,.12);border-color:rgba(255,70,70,.25);color:rgba(255,140,140,.85)}
.sg-body{flex:1;overflow-y:auto;padding:20px 22px;display:flex;flex-direction:column;gap:14px}
.sg-field{display:flex;flex-direction:column;gap:6px}
.sg-label{font-size:11px;font-weight:700;letter-spacing:.6px;text-transform:uppercase;color:rgba(255,255,255,.45)}
.sg-input{width:100%;padding:10px 14px;border-radius:10px;border:1px solid rgba(255,255,255,.09);background:rgba(255,255,255,.04);color:rgba(255,255,255,.85);font-size:13px;outline:none;box-sizing:border-box}
.sg-input:focus{border-color:rgba(var(--theme-surface-tint-rgb),.45);box-shadow:0 0 0 3px rgba(var(--theme-surface-tint-rgb),.06)}
.sg-input--flex{flex:1;min-width:0}
.sg-row{display:flex;gap:8px}
.sg-footer{display:flex;justify-content:flex-end;gap:12px;padding:14px 22px 18px;border-top:1px solid rgba(255,255,255,.07)}
.sg-btn{height:38px;padding:0 20px;border-radius:10px;border:1px solid rgba(255,255,255,.10);font-size:13px;font-weight:600;cursor:pointer;background:rgba(255,255,255,.05);color:rgba(255,255,255,.70)}
.sg-btn--outline{height:auto;padding:10px 14px;white-space:nowrap}
.sg-btn--cancel{background:rgba(220,70,70,.10);border-color:rgba(220,70,70,.18);color:rgba(240,120,120,.82)}
.sg-btn--cancel:hover{background:rgba(220,70,70,.16);border-color:rgba(220,70,70,.28);color:rgba(250,140,140,.92)}
.sg-btn--confirm{background:rgba(70,200,120,.12);border-color:rgba(70,200,120,.22);color:rgba(120,240,160,.88)}
.sg-btn--confirm:hover{background:rgba(70,200,120,.18);border-color:rgba(70,200,120,.34);color:rgba(130,250,170,.95)}
</style>
