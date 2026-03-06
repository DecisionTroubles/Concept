<script setup lang="ts">
import { computed } from 'vue'
import type { Node, NoteType } from '@/bindings'
import NodeBlockRenderer from '@/components/node/NodeBlockRenderer.vue'
import {
  blocksFromLegacyPage,
  inferSummaryBlocks,
  parseLayout,
  parseSchemaFields,
  type LayoutBlock,
} from '@/components/node/layout'

const props = defineProps<{
  node: Node
  noteType: NoteType | null
}>()

const fieldByKey = computed(() => {
  const map = new Map<string, { key: string; label?: string; widget?: string }>()
  for (const field of parseSchemaFields(props.noteType)) map.set(field.key, field)
  return map
})

const summaryBlocks = computed<LayoutBlock[]>(() => {
  const layout = parseLayout(props.noteType)
  const explicit = layout.summary?.blocks
  if (Array.isArray(explicit) && explicit.length > 0) return explicit.map(block => ({ ...block, compact: true }))

  const overviewPage = (layout.pages ?? []).find(page => page.id === 'overview') ?? (layout.pages ?? [])[0]
  if (overviewPage) {
    if (overviewPage.blocks?.length) return overviewPage.blocks.map(block => ({ ...block, compact: true })).slice(0, 3)
    const legacyBlocks = blocksFromLegacyPage(overviewPage)
    if (legacyBlocks.length > 0) return legacyBlocks.map(block => ({ ...block, compact: true }))
  }

  return inferSummaryBlocks(props.node, fieldByKey.value)
})
</script>

<template>
  <div class="summary-renderer">
    <NodeBlockRenderer
      v-for="(block, index) in summaryBlocks"
      :key="`${block.type || 'field_group'}-${index}-${block.label || block.field || 'block'}`"
      :node="node"
      :note-type="noteType"
      :block="block"
      compact
    />
  </div>
</template>

<style scoped>
.summary-renderer {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>
