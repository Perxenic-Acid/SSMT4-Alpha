<script setup lang="ts">
import { computed } from 'vue';
import type { ModKeyInfo } from './ModsManagement.types';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
  visible: boolean;
  modId: string;
  loading: boolean;
  error: string;
  items: ModKeyInfo[];
  anchorRect: { top: number; left: number; width: number; height: number } | null;
  getModKeySectionTitle: (item: ModKeyInfo) => string;
  getModKeyDisplayName: (item: ModKeyInfo) => string;
}>();

const emit = defineEmits<{
  keepalive: [];
  scheduleclose: [];
}>();

const panelStyle = computed(() => {
  if (!props.anchorRect) return { display: 'none' };

  const gap = 8;
  const maxW = window.innerWidth * 0.8;
  const maxH = window.innerHeight * 0.8;

  // Default: below anchor (K badge is at top-right of card)
  let left = props.anchorRect.left;
  let top = props.anchorRect.top + props.anchorRect.height + gap;

  // If bottom overflows, try above anchor
  if (top + maxH > window.innerHeight - gap) {
    top = props.anchorRect.top - maxH - gap;
    // If top overflows too, stick to bottom with scroll
    if (top < gap) {
      top = gap;
    }
  }

  // If right overflows, right-align with anchor
  if (left + maxW > window.innerWidth - gap) {
    left = window.innerWidth - gap - maxW;
  }

  // Ensure left edge doesn't go off screen
  if (left < gap) {
    left = gap;
  }

  return {
    left: `${left}px`,
    top: `${top}px`,
    maxWidth: `${maxW}px`,
    maxHeight: `${maxH}px`,
  };
});
</script>

<template>
  <Teleport to="body">
    <Transition name="sk-float">
      <div
        v-if="visible && anchorRect"
        class="sk-float-panel"
        :style="panelStyle"
        @mouseenter="emit('keepalive')"
        @mouseleave="emit('scheduleclose')"
      >
        <!-- Header -->
        <div class="skf-header">
          <span class="skf-title">{{ t('modsManagement.ui.modKeyListTitle') }}</span>
        </div>

        <!-- Body -->
        <div class="skf-body glass-scrollbar--thin">
          <!-- Loading -->
          <div v-if="loading" class="skf-state">{{ t('modsManagement.ui.loadingModKeys') }}</div>
          <!-- Error -->
          <div v-else-if="error" class="skf-state is-error">{{ t('modsManagement.messages.loadModKeyListFailed', { error }) }}</div>
          <!-- Empty -->
          <div v-else-if="!items.length" class="skf-state">{{ t('modsManagement.ui.noModKeys') }}</div>
          <!-- Key List -->
          <div v-else class="skf-list">
            <div
              v-for="(item, index) in items"
              :key="`${modId}-key-${index}-${item.id}`"
              class="skf-item"
            >
              <div class="skf-item-info">
                <div class="skf-item-name">{{ getModKeyDisplayName(item) }}</div>
                <div class="skf-item-desc">{{ getModKeySectionTitle(item) }}</div>
              </div>
              <div class="skf-item-meta">
                <span v-if="item.keyType" class="skf-type">{{ item.keyType }}</span>
                <span class="skf-source">{{ item.sourceIni }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* ═══ Floating panel — white glass ═══ */
.sk-float-enter-active,
.sk-float-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.sk-float-enter-from,
.sk-float-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.96);
}

.sk-float-panel {
  position: fixed;
  z-index: 11000;
  display: flex;
  flex-direction: column;
  background: var(--t-material-bg);
  backdrop-filter: blur(28px) saturate(1.6);
  -webkit-backdrop-filter: blur(28px) saturate(1.6);
  border: var(--t-material-border);
  border-radius: 14px;
  box-shadow: var(--t-material-shadow);
  overflow: hidden;
  min-width: 220px;
}

/* ═══ Header ═══ */
.skf-header {
  flex-shrink: 0;
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.skf-title {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.8px;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.70);
}

/* ═══ Body (scrollable) ═══ */
.skf-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px 10px;
}

/* ═══ States ═══ */
.skf-state {
  padding: 24px 14px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.50);
  text-align: center;
}
.skf-state.is-error { color: rgba(255, 150, 150, 0.80); }

/* ═══ List ═══ */
.skf-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* ═══ Item ═══ */
.skf-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  transition: background 0.15s, border-color 0.15s;
}
.skf-item:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.15);
}

.skf-item-info {
  flex: 1;
  min-width: 0;
}

.skf-item-name {
  font-size: 12px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.90);
  word-break: break-word;
}

.skf-item-desc {
  margin-top: 2px;
  font-size: 10px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.40);
  word-break: break-word;
  font-family: 'Consolas', 'Courier New', monospace;
}

.skf-item-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 3px;
  flex-shrink: 0;
}

.skf-type {
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(var(--theme-surface-tint-rgb), 0.08);
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.12);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: rgba(var(--theme-surface-tint-rgb), 0.65);
}

.skf-source {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.25);
}
</style>
