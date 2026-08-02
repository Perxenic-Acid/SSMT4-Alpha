<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { renderReleaseNotesMarkdown } from '../utils/ReleaseNotesMarkdown'

const props = defineProps<{
  title: string
  message: string
  releaseNotesTitle: string
  releaseNotes: string
  confirmText: string
  cancelText: string
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const confirmButtonRef = ref<HTMLButtonElement | null>(null)

const renderedReleaseNotes = computed(() => renderReleaseNotesMarkdown(props.releaseNotes))

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    emit('cancel')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  void nextTick(() => {
    confirmButtonRef.value?.focus()
  })
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="app-update-dialog-overlay" @click.self="emit('cancel')">
    <section
      class="app-update-dialog"
      role="dialog"
      aria-modal="true"
      :aria-label="title"
    >
      <header class="app-update-dialog__header">
        <div>
          <h2 class="app-update-dialog__title">{{ title }}</h2>
        </div>
        <button
          type="button"
          class="app-update-dialog__close"
          aria-label="Close"
          @click="emit('cancel')"
        >
          ×
        </button>
      </header>

      <div class="app-update-dialog__body">
        <p class="app-update-dialog__message">{{ message }}</p>

        <div class="app-update-dialog__release">
          <div class="app-update-dialog__release-title">{{ releaseNotesTitle }}</div>
          <div class="app-update-dialog__release-notes" v-html="renderedReleaseNotes"></div>
        </div>
      </div>

      <footer class="app-update-dialog__footer">
        <button
          type="button"
          class="app-update-dialog__button app-update-dialog__button--ghost"
          @click="emit('cancel')"
        >
          {{ cancelText }}
        </button>
        <button
          ref="confirmButtonRef"
          type="button"
          class="app-update-dialog__button app-update-dialog__button--primary"
          @click="emit('confirm')"
        >
          {{ confirmText }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.app-update-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: grid;
  place-items: center;
  padding: 32px;
  background:
    radial-gradient(circle at 50% 42%, rgba(var(--theme-surface-tint-rgb), 0.16), transparent 46%),
    rgba(3, 5, 10, 0.78);
  backdrop-filter: blur(8px) saturate(1.1);
  -webkit-backdrop-filter: blur(8px) saturate(1.1);
}

.app-update-dialog {
  width: min(620px, calc(100vw - 48px));
  max-height: min(78vh, 680px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  color: rgba(255, 255, 255, 0.92);
  background: var(--t-material-bg);
  border: var(--t-material-border);
  border-radius: 16px;
  box-shadow: var(--t-material-shadow);
}

.app-update-dialog__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
  padding: 24px 26px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.035);
}

.app-update-dialog__title {
  margin: 0;
  color: rgba(255, 255, 255, 0.96);
  font-size: 22px;
  font-weight: 750;
  line-height: 1.25;
}

.app-update-dialog__close {
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  color: rgba(255, 255, 255, 0.64);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.10);
  border-radius: 8px;
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
  transition: color 0.16s ease, background 0.16s ease, border-color 0.16s ease;
}

.app-update-dialog__close:hover,
.app-update-dialog__close:focus-visible {
  color: rgba(255, 255, 255, 0.95);
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.22);
  outline: none;
}

.app-update-dialog__body {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 22px 26px 24px;
}

.app-update-dialog__message {
  margin: 0;
  color: rgba(255, 255, 255, 0.82);
  font-size: 15px;
  font-weight: 650;
  line-height: 1.65;
}

.app-update-dialog__release {
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--t-surface-soft);
  border: 1px solid rgba(255, 255, 255, 0.10);
  border-radius: 12px;
}

.app-update-dialog__release-title {
  padding: 12px 16px;
  color: rgba(255, 255, 255, 0.72);
  font-size: 13px;
  font-weight: 700;
  line-height: 1.3;
  background: rgba(255, 255, 255, 0.045);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.app-update-dialog__release-notes {
  min-height: 96px;
  max-height: 300px;
  margin: 0;
  padding: 15px 16px 16px;
  overflow: auto;
  color: rgba(255, 255, 255, 0.80);
  font-family: inherit;
  font-size: 14px;
  line-height: 1.65;
  word-break: break-word;
}

.app-update-dialog__release-notes :deep(p) {
  margin: 0 0 10px;
}

.app-update-dialog__release-notes :deep(p:last-child),
.app-update-dialog__release-notes :deep(ul:last-child),
.app-update-dialog__release-notes :deep(ol:last-child) {
  margin-bottom: 0;
}

.app-update-dialog__release-notes :deep(ul),
.app-update-dialog__release-notes :deep(ol) {
  margin: 0 0 12px;
  padding-left: 22px;
}

.app-update-dialog__release-notes :deep(li) {
  margin: 0 0 8px;
}

.app-update-dialog__release-notes :deep(h3),
.app-update-dialog__release-notes :deep(h4),
.app-update-dialog__release-notes :deep(h5) {
  margin: 14px 0 8px;
  color: rgba(255, 255, 255, 0.92);
  font-size: 15px;
  line-height: 1.4;
}

.app-update-dialog__release-notes :deep(a) {
  color: rgba(150, 205, 255, 0.95);
  text-decoration: none;
}

.app-update-dialog__release-notes :deep(a:hover) {
  text-decoration: underline;
}

.app-update-dialog__release-notes :deep(code) {
  padding: 2px 5px;
  color: rgba(255, 255, 255, 0.92);
  background: rgba(255, 255, 255, 0.10);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 5px;
  font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
  font-size: 0.92em;
}

.app-update-dialog__release-notes :deep(img) {
  display: block;
  max-width: 100%;
  height: auto;
  margin: 10px 0 12px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.10);
  background: rgba(255, 255, 255, 0.04);
}

.app-update-dialog__release-notes::-webkit-scrollbar {
  width: 8px;
}

.app-update-dialog__release-notes::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.04);
}

.app-update-dialog__release-notes::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.18);
  border-radius: 8px;
}

.app-update-dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 26px 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.12);
}

.app-update-dialog__button {
  min-width: 104px;
  height: 38px;
  padding: 0 18px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.16s ease, background 0.16s ease, border-color 0.16s ease, color 0.16s ease;
}

.app-update-dialog__button:hover {
  transform: translateY(-1px);
}

.app-update-dialog__button:focus-visible {
  outline: 2px solid rgba(255, 255, 255, 0.42);
  outline-offset: 2px;
}

.app-update-dialog__button--ghost {
  color: rgba(255, 255, 255, 0.72);
  background: rgba(255, 255, 255, 0.07);
  border: 1px solid rgba(255, 255, 255, 0.12);
}

.app-update-dialog__button--ghost:hover {
  color: rgba(255, 255, 255, 0.92);
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.24);
}

.app-update-dialog__button--primary {
  color: rgba(14, 18, 26, 0.96);
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.98);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.24);
}

.app-update-dialog__button--primary:hover {
  background: #ffffff;
  border-color: #ffffff;
}

@media (max-width: 640px) {
  .app-update-dialog-overlay {
    padding: 18px;
  }

  .app-update-dialog {
    width: 100%;
    max-height: calc(100vh - 36px);
  }

  .app-update-dialog__header,
  .app-update-dialog__body {
    padding-left: 20px;
    padding-right: 20px;
  }

  .app-update-dialog__footer {
    padding-left: 20px;
    padding-right: 20px;
  }
}
</style>
