<script setup lang="ts">
import { computed } from 'vue'
import type { Node, NoteType } from '@/bindings'
import NodeFieldRenderer from '@/components/node/NodeFieldRenderer.vue'
import NodeBlockRenderer from '@/components/node/NodeBlockRenderer.vue'
import {
  blocksFromLegacyPage,
  inferFallbackBlocks,
  parseLayout,
  parseSchemaFields,
  type LayoutPage,
  type NoteFieldDefinition,
} from '@/components/node/layout'

const props = defineProps<{
  node: Node
  noteType: NoteType | null
  activePageId?: string | null
}>()

const schemaFields = computed<NoteFieldDefinition[]>(() => {
  return parseSchemaFields(props.noteType)
})

const fieldByKey = computed(() => {
  const map = new Map<string, NoteFieldDefinition>()
  for (const field of schemaFields.value) map.set(field.key, field)
  return map
})

const pages = computed<LayoutPage[]>(() => {
  const parsed = parseLayout(props.noteType)
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

const fallbackBlocks = computed(() => inferFallbackBlocks(props.node, fieldByKey.value))
</script>

<template>
  <div v-if="hasLayout" class="note-type-pages">
    <section v-for="page in visiblePages" :key="page.id" class="note-page">
      <div v-if="page.blocks?.length" class="note-block-list">
        <NodeBlockRenderer
          v-for="(block, index) in page.blocks"
          :key="`${page.id}-block-${index}-${block.type || 'field_group'}`"
          :node="node"
          :note-type="noteType"
          :block="block"
        />
      </div>
      <div v-else-if="(page.sections?.length || 0) > 0" class="note-sections">
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
      <div v-else class="note-block-list">
        <NodeBlockRenderer
          v-for="(block, index) in blocksFromLegacyPage(page)"
          :key="`${page.id}-fallback-${index}`"
          :node="node"
          :note-type="noteType"
          :block="block"
        />
      </div>
    </section>
  </div>

  <div v-else class="note-type-pages">
    <section class="note-page">
      <div class="note-block-list" v-if="fallbackBlocks.length > 0">
        <NodeBlockRenderer
          v-for="(block, index) in fallbackBlocks"
          :key="`fallback-block-${index}`"
          :node="node"
          :note-type="noteType"
          :block="block"
        />
      </div>
      <div v-else class="note-field-list">
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

.note-sections {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.note-block-list {
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
