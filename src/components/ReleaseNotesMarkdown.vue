<script setup lang="ts">
import { computed } from 'vue'
import { renderReleaseNotesMarkdown } from '../utils/ReleaseNotesMarkdown'

const props = defineProps<{
  content: string
  compact?: boolean
}>()

const renderedContent = computed(() => renderReleaseNotesMarkdown(props.content || ''))
</script>

<template>
  <div
    class="release-notes-markdown"
    :class="{ 'release-notes-markdown--compact': compact }"
    v-html="renderedContent"
  ></div>
</template>

<style scoped>
.release-notes-markdown {
  color: inherit;
  font: inherit;
  line-height: inherit;
  word-break: break-word;
}

.release-notes-markdown :deep(p) {
  margin: 0 0 10px;
}

.release-notes-markdown :deep(p:last-child),
.release-notes-markdown :deep(ul:last-child),
.release-notes-markdown :deep(ol:last-child) {
  margin-bottom: 0;
}

.release-notes-markdown :deep(ul),
.release-notes-markdown :deep(ol) {
  margin: 0 0 12px;
  padding-left: 22px;
}

.release-notes-markdown :deep(li) {
  margin: 0 0 8px;
}

.release-notes-markdown--compact :deep(p) {
  margin-bottom: 6px;
}

.release-notes-markdown--compact :deep(ul),
.release-notes-markdown--compact :deep(ol) {
  margin-bottom: 8px;
  padding-left: 18px;
}

.release-notes-markdown--compact :deep(li) {
  margin-bottom: 5px;
}

.release-notes-markdown :deep(h3),
.release-notes-markdown :deep(h4),
.release-notes-markdown :deep(h5) {
  margin: 14px 0 8px;
  color: rgba(255, 255, 255, 0.92);
  font-size: 1.05em;
  line-height: 1.4;
}

.release-notes-markdown :deep(a) {
  color: rgba(150, 205, 255, 0.95);
  text-decoration: none;
}

.release-notes-markdown :deep(a:hover) {
  text-decoration: underline;
}

.release-notes-markdown :deep(code) {
  padding: 2px 5px;
  color: rgba(255, 255, 255, 0.92);
  background: rgba(255, 255, 255, 0.10);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 5px;
  font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
  font-size: 0.92em;
}

.release-notes-markdown :deep(img) {
  display: block;
  max-width: 100%;
  height: auto;
  margin: 10px 0 12px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.10);
  background: rgba(255, 255, 255, 0.04);
}

.release-notes-markdown--compact :deep(img) {
  margin: 8px 0 10px;
}
</style>
