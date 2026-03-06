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
    <div v-if="widget === 'long_text'" class="node-field-value node-field-value-long">
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
</style>
