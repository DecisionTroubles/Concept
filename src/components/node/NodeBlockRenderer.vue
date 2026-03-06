<script setup lang="ts">
import { computed } from 'vue'
import type { Node, NoteType } from '@/bindings'
import { useGraphStore } from '@/stores/graph'
import {
  fieldLabel,
  fieldValue,
  parseSchemaFields,
  type LayoutBlock,
  type NoteFieldDefinition,
} from '@/components/node/layout'

const props = defineProps<{
  node: Node
  noteType: NoteType | null
  block: LayoutBlock
  compact?: boolean
}>()

const graphStore = useGraphStore()

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
}

function renderInlineMarkdown(value: string): string {
  return escapeHtml(value)
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
}

function markdownToHtml(value: string): string {
  const lines = value.split(/\r?\n/)
  const html: string[] = []
  let inList = false

  for (const rawLine of lines) {
    const line = rawLine.trim()
    if (!line) {
      if (inList) {
        html.push('</ul>')
        inList = false
      }
      continue
    }
    if (line.startsWith('- ') || line.startsWith('* ')) {
      if (!inList) {
        html.push('<ul>')
        inList = true
      }
      html.push(`<li>${renderInlineMarkdown(line.slice(2))}</li>`)
      continue
    }
    if (inList) {
      html.push('</ul>')
      inList = false
    }
    if (line.startsWith('### ')) {
      html.push(`<h4>${renderInlineMarkdown(line.slice(4))}</h4>`)
      continue
    }
    if (line.startsWith('## ')) {
      html.push(`<h3>${renderInlineMarkdown(line.slice(3))}</h3>`)
      continue
    }
    if (line.startsWith('# ')) {
      html.push(`<h2>${renderInlineMarkdown(line.slice(2))}</h2>`)
      continue
    }
    html.push(`<p>${renderInlineMarkdown(line)}</p>`)
  }

  if (inList) html.push('</ul>')
  return html.join('')
}

const schemaFields = computed<NoteFieldDefinition[]>(() => parseSchemaFields(props.noteType))

const fieldByKey = computed(() => {
  const map = new Map<string, NoteFieldDefinition>()
  for (const field of schemaFields.value) map.set(field.key, field)
  return map
})

const effectiveCompact = computed(() => props.compact || props.block.compact)
const blockType = computed(() => props.block.type || 'field_group')
const toneClass = computed(() => `tone-${props.block.tone || 'info'}`)
const primaryField = computed(() => props.block.field || '')
const fieldKeys = computed(() => {
  if (props.block.fields?.length) return props.block.fields
  if (props.block.field) return [props.block.field]
  return []
})
const markdownHtml = computed(() => markdownToHtml(fieldValue(props.node, primaryField.value)))
const codeValue = computed(() => fieldValue(props.node, primaryField.value))
const imageSrc = computed(() => fieldValue(props.node, primaryField.value))
const imageCaption = computed(() => props.block.caption_field ? fieldValue(props.node, props.block.caption_field) : '')
const relationItems = computed(() =>
  props.node.connections.slice(0, effectiveCompact.value ? 4 : 8).map(conn => ({
    id: conn.id,
    targetId: conn.target_id,
    title: graphStore.nodes.find(node => node.id === conn.target_id)?.title ?? conn.target_id,
    edgeType: conn.edge_type,
  }))
)
</script>

<template>
  <article :class="['node-block', `node-block-${blockType}`, { compact: effectiveCompact }]">
    <div v-if="block.label" class="node-block-label">{{ block.label }}</div>

    <template v-if="blockType === 'field_group'">
      <div class="field-group-list">
        <div v-for="key in fieldKeys" :key="key" class="field-entry">
          <div class="field-entry-label">{{ fieldLabel(fieldByKey, key) }}</div>
          <div class="field-entry-value" :class="{ compact: effectiveCompact }">
            {{ fieldValue(node, key) || 'No value yet.' }}
          </div>
        </div>
      </div>
    </template>

    <template v-else-if="blockType === 'markdown'">
      <div class="markdown-block" v-html="markdownHtml || '<p>No value yet.</p>'" />
    </template>

    <template v-else-if="blockType === 'code'">
      <div class="code-head">
        <span v-if="block.language" class="code-language">{{ block.language }}</span>
      </div>
      <pre class="code-block"><code>{{ codeValue || 'No code yet.' }}</code></pre>
    </template>

    <template v-else-if="blockType === 'image' || blockType === 'diagram'">
      <div v-if="imageSrc" class="image-wrap">
        <img :src="imageSrc" :alt="block.label || primaryField || 'Node visual'" class="image-block" />
        <div v-if="imageCaption" class="image-caption">{{ imageCaption }}</div>
      </div>
      <div v-else class="empty-block">No image yet.</div>
    </template>

    <template v-else-if="blockType === 'callout'">
      <div :class="['callout-block', toneClass]">
        {{ fieldValue(node, primaryField) || 'No value yet.' }}
      </div>
    </template>

    <template v-else-if="blockType === 'relations'">
      <div v-if="relationItems.length > 0" class="relations-list">
        <button
          v-for="item in relationItems"
          :key="item.id"
          class="relation-item"
          @click="graphStore.requestFocus(item.targetId)"
        >
          <span>{{ item.title }}</span>
          <small>{{ item.edgeType }}</small>
        </button>
      </div>
      <div v-else class="empty-block">No relations yet.</div>
    </template>

    <template v-else>
      <div class="empty-block">Unsupported block type: {{ blockType }}</div>
    </template>
  </article>
</template>

<style scoped>
.node-block {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.035);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.node-block.compact {
  padding: 10px;
  gap: 8px;
}

.node-block-label,
.field-entry-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.field-group-list,
.relations-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field-entry {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-entry-value,
.empty-block,
.image-caption,
.callout-block {
  font-size: 13px;
  line-height: 1.55;
  color: #d6dae6;
}

.field-entry-value {
  white-space: pre-wrap;
}

.field-entry-value.compact {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.markdown-block :deep(h2),
.markdown-block :deep(h3),
.markdown-block :deep(h4) {
  margin: 0;
  color: var(--app-text-primary);
}

.markdown-block :deep(p),
.markdown-block :deep(ul) {
  margin: 0;
  color: #d6dae6;
  line-height: 1.55;
}

.markdown-block :deep(ul) {
  padding-left: 18px;
}

.markdown-block :deep(code) {
  font-family: ui-monospace, 'Cascadia Code', monospace;
  color: var(--app-accent);
}

.code-head {
  display: flex;
  justify-content: flex-end;
}

.code-language {
  font-size: 10px;
  color: var(--app-accent);
  border-radius: 999px;
  padding: 3px 7px;
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 26%, transparent);
}

.code-block {
  margin: 0;
  padding: 14px;
  border-radius: 12px;
  overflow: auto;
  background: rgba(8, 12, 22, 0.9);
  border: 1px solid rgba(125, 145, 185, 0.15);
  color: #d9e7ff;
  font-size: 12px;
  line-height: 1.6;
  font-family: ui-monospace, 'Cascadia Code', monospace;
}

.image-wrap {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.image-block {
  display: block;
  width: 100%;
  max-height: 320px;
  object-fit: contain;
  border-radius: 12px;
  background: rgba(0, 0, 0, 0.14);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.callout-block {
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid transparent;
}

.tone-info {
  background: rgba(91, 143, 255, 0.12);
  border-color: rgba(91, 143, 255, 0.26);
}

.tone-tip {
  background: rgba(61, 214, 140, 0.12);
  border-color: rgba(61, 214, 140, 0.26);
}

.tone-warning {
  background: rgba(245, 158, 11, 0.12);
  border-color: rgba(245, 158, 11, 0.26);
}

.tone-danger {
  background: rgba(244, 63, 94, 0.12);
  border-color: rgba(244, 63, 94, 0.26);
}

.relation-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
  color: var(--app-text-primary);
  cursor: pointer;
}

.relation-item small {
  font-size: 10px;
  color: var(--app-text-secondary);
}
</style>
