<script setup lang="ts">
import { computed } from 'vue'
import type { Node, NoteType } from '@/bindings'
import NodeFieldRenderer from '@/components/node/NodeFieldRenderer.vue'

type NoteFieldDefinition = {
  key: string
  label?: string
  type?: string
  widget?: string
}

type LayoutItem = {
  field?: string
}

type LayoutSection = {
  id: string
  label?: string
  items?: LayoutItem[]
}

type LayoutPage = {
  id: string
  label?: string
  sections?: LayoutSection[]
}

const props = defineProps<{
  node: Node
  noteType: NoteType | null
  activePageId?: string | null
}>()

function parseJson<T>(raw: string | null | undefined, fallback: T): T {
  if (!raw) return fallback
  try {
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

const schemaFields = computed<NoteFieldDefinition[]>(() => {
  if (!props.noteType) return []
  const parsed = parseJson<{ fields?: NoteFieldDefinition[] }>(props.noteType.schema_json, {})
  return Array.isArray(parsed.fields) ? parsed.fields : []
})

const fieldByKey = computed(() => {
  const map = new Map<string, NoteFieldDefinition>()
  for (const field of schemaFields.value) map.set(field.key, field)
  return map
})

const pages = computed<LayoutPage[]>(() => {
  if (!props.noteType) return []
  const parsed = parseJson<{ pages?: LayoutPage[] }>(props.noteType.layout_json, {})
  return Array.isArray(parsed.pages) ? parsed.pages : []
})

const visiblePages = computed(() => {
  if (!props.activePageId) return pages.value
  return pages.value.filter(page => page.id === props.activePageId)
})

const hasLayout = computed(() => pages.value.length > 0)

const fallbackFields = computed<NoteFieldDefinition[]>(() => {
  const explicit = Object.keys(props.node.note_fields).map(key => ({
    key,
    label: key,
    widget: key.toLowerCase().includes('example') ? 'long_text' : 'text',
  }))
  if (explicit.length > 0) return explicit
  return [{ key: 'content_data', label: 'Content', widget: 'long_text' }]
})
</script>

<template>
  <div v-if="hasLayout" class="note-type-pages">
    <section v-for="page in visiblePages" :key="page.id" class="note-page">
      <div class="note-page-title">{{ page.label || page.id }}</div>
      <div class="note-sections">
        <article v-for="section in page.sections || []" :key="section.id" class="note-section">
          <div v-if="section.label" class="note-section-title">{{ section.label }}</div>
          <div class="note-field-list">
            <NodeFieldRenderer
              v-for="item in section.items || []"
              :key="`${page.id}-${section.id}-${item.field}`"
              :node="node"
              :field="fieldByKey.get(item.field || '') || { key: item.field || 'content_data', label: item.field || 'Content', widget: 'text' }"
            />
          </div>
        </article>
      </div>
    </section>
  </div>

  <div v-else class="note-type-pages">
    <section class="note-page">
      <div class="note-page-title">{{ noteType?.name || 'Content' }}</div>
      <div class="note-field-list">
        <NodeFieldRenderer v-for="field in fallbackFields" :key="field.key" :node="node" :field="field" />
      </div>
    </section>
  </div>
</template>

<style scoped>
.note-type-pages {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.note-page {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.note-page-title {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-accent);
}

.note-sections {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.note-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.note-section-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--app-text-primary);
}

.note-field-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>
