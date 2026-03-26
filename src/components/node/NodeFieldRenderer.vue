<script setup lang="ts">
import { computed } from 'vue'
import type { Node } from '@/bindings'

type NoteFieldDefinition = {
  key: string
  label?: string
  type?: string
  widget?: string
}

const props = defineProps<{
  node: Node
  field: NoteFieldDefinition
}>()

const label = computed(() => props.field.label || props.field.key)

const value = computed(() => {
  const key = props.field.key
  if (key in props.node.note_fields) return props.node.note_fields[key] ?? ''
  if (key === 'Content' || key === 'content' || key === 'content_data') return props.node.content_data ?? ''
  if (key === 'title') return props.node.title
  return ''
})

const widget = computed(() => props.field.widget || 'text')
</script>

<template>
  <div class="node-field">
    <div class="node-field-label">{{ label }}</div>
    <div
      v-if="widget === 'html'"
      class="node-field-value node-field-value-html"
      v-html="value || '<p>No value yet.</p>'"
    />
    <div v-else-if="widget === 'long_text'" class="node-field-value node-field-value-long">
      {{ value || 'No value yet.' }}
    </div>
    <div v-else class="node-field-value">
      {{ value || 'No value yet.' }}
    </div>
  </div>
</template>

<style scoped>
.node-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.node-field-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.node-field-value {
  font-size: 13px;
  line-height: 1.5;
  color: #d6dae6;
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  min-height: 42px;
}

.node-field-value-long {
  white-space: pre-wrap;
}

.node-field-value-html {
  white-space: normal;
  overflow-x: auto;
}

.node-field-value-html :deep(section) {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0 0 16px;
}

.node-field-value-html :deep(h1),
.node-field-value-html :deep(h2),
.node-field-value-html :deep(h3),
.node-field-value-html :deep(h4) {
  margin: 0;
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.node-field-value-html :deep(p),
.node-field-value-html :deep(ul),
.node-field-value-html :deep(ol) {
  margin: 0;
}

.node-field-value-html :deep(img) {
  display: block;
  max-width: 100%;
  height: auto;
  border-radius: 12px;
}

.node-field-value-html :deep(audio),
.node-field-value-html :deep(video) {
  width: 100%;
  max-width: 360px;
}
</style>
