<script setup lang="ts">
import { computed } from 'vue'
import type { Node, NoteType } from '@/bindings'
import NodeBlockRenderer from '@/components/node/NodeBlockRenderer.vue'
import {
  blocksFromLegacyPage,
  inferFallbackContentPages,
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

const schemaFields = computed<NoteFieldDefinition[]>(() => parseSchemaFields(props.noteType))

const fieldByKey = computed(() => {
  const map = new Map<string, NoteFieldDefinition>()
  for (const field of schemaFields.value) map.set(field.key, field)
  return map
})

const pages = computed<LayoutPage[]>(() => {
  const parsed = parseLayout(props.noteType)
  const authored = Array.isArray(parsed.pages) ? parsed.pages.filter(page => (page.kind ?? 'content') === 'content') : []
  if (authored.length > 0) return authored
  return inferFallbackContentPages(props.node, fieldByKey.value)
})

const visiblePages = computed(() => {
  if (!props.activePageId) return pages.value
  return pages.value.filter(page => page.id === props.activePageId)
})

function pagePreset(page: LayoutPage): 'overview' | 'example' | 'generic' {
  const id = page.id.toLowerCase()
  if (id === 'overview') return 'overview'
  if (id === 'example') return 'example'
  return 'generic'
}
</script>

<template>
  <div class="note-type-pages">
    <section
      v-for="page in visiblePages"
      :key="page.id"
      class="note-page"
      :class="`note-page-${pagePreset(page)}`"
    >
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
          <div class="note-block-list">
            <NodeBlockRenderer
              v-for="(block, index) in blocksFromLegacyPage({ ...page, sections: [{ ...section }] })"
              :key="`${page.id}-${section.id}-${index}`"
              :node="node"
              :note-type="noteType"
              :block="block"
            />
          </div>
        </article>
      </div>
      <div v-else class="note-empty">No page content yet.</div>
    </section>
  </div>
</template>

<style scoped>
.note-type-pages {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.note-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.note-block-list {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 16px;
}

.note-sections {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.note-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.note-section-title {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.note-empty {
  color: var(--app-text-secondary);
  font-size: 0.95rem;
}

.note-page-overview :deep(.node-block-field_group:first-child) {
  padding: 22px 24px;
}

.note-page-overview :deep(.node-block-callout),
.note-page-example :deep(.node-block-callout) {
  max-width: 48rem;
}

.note-page-example :deep(.node-block-code),
.note-page-example :deep(.node-block-markdown),
.note-page-example :deep(.node-block-image),
.note-page-example :deep(.node-block-diagram) {
  width: 100%;
}

@media (min-width: 1080px) {
  .note-page-overview .note-block-list,
  .note-page-example .note-block-list {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
