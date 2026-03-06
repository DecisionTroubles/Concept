import type { Node, NoteType } from '@/bindings'

export type NoteFieldDefinition = {
  key: string
  label?: string
  type?: string
  widget?: string
}

export type LayoutItem = {
  field?: string
}

export type LayoutSection = {
  id: string
  label?: string
  items?: LayoutItem[]
}

export type LayoutBlock = {
  type?: 'field_group' | 'markdown' | 'code' | 'image' | 'diagram' | 'callout' | 'relations' | 'extension'
  label?: string
  field?: string
  fields?: string[]
  tone?: 'info' | 'tip' | 'warning' | 'danger'
  language?: string
  caption_field?: string
  extension_id?: string
  compact?: boolean
}

export type LayoutPage = {
  id: string
  label?: string
  kind?: 'content' | 'built_in' | 'extension'
  source?: string
  slot?: string
  extension_id?: string
  sections?: LayoutSection[]
  blocks?: LayoutBlock[]
}

export type LayoutRoot = {
  version?: number
  pages?: LayoutPage[]
  summary?: {
    blocks?: LayoutBlock[]
  }
}

export function parseJson<T>(raw: string | null | undefined, fallback: T): T {
  if (!raw) return fallback
  try {
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

export function parseSchemaFields(noteType: NoteType | null): NoteFieldDefinition[] {
  if (!noteType) return []
  const parsed = parseJson<{ fields?: NoteFieldDefinition[] }>(noteType.schema_json, {})
  return Array.isArray(parsed.fields) ? parsed.fields : []
}

export function parseLayout(noteType: NoteType | null): LayoutRoot {
  if (!noteType) return {}
  return parseJson<LayoutRoot>(noteType.layout_json, {})
}

export function fieldValue(node: Node, key: string): string {
  if (key in node.note_fields) return node.note_fields[key] ?? ''
  if (key === 'Content' || key === 'content' || key === 'content_data') return node.content_data ?? ''
  if (key === 'title') return node.title
  return ''
}

export function fieldLabel(fieldByKey: Map<string, NoteFieldDefinition>, key: string): string {
  return fieldByKey.get(key)?.label || key
}

export function blocksFromLegacyPage(page: LayoutPage): LayoutBlock[] {
  const fields = (page.sections ?? [])
    .flatMap(section => (section.items ?? []).map(item => item.field ?? '').filter(Boolean))
  if (fields.length === 0) return []
  return [{
    type: 'field_group',
    label: page.label || 'Content',
    fields,
  }]
}

export function inferFallbackBlocks(node: Node, fieldByKey: Map<string, NoteFieldDefinition>): LayoutBlock[] {
  const keys = Object.keys(node.note_fields)
  if (keys.length === 0) {
    return [{ type: 'field_group', label: 'Content', fields: ['content_data'] }]
  }

  const summaryKeys = keys.filter(key => {
    const lower = key.toLowerCase()
    return lower.includes('summary') || lower.includes('meaning') || lower.includes('function') || lower.includes('term') || lower.includes('phrase') || lower.includes('pattern')
  })
  const codeField = keys.find(key => (fieldByKey.get(key)?.widget || '').toLowerCase() === 'code')
  const imageField = keys.find(key => {
    const widget = (fieldByKey.get(key)?.widget || '').toLowerCase()
    return widget === 'image' || widget === 'diagram'
  })
  const calloutField = keys.find(key => {
    const lower = key.toLowerCase()
    return lower.includes('tip') || lower.includes('pitfall') || lower.includes('warning') || lower.includes('note')
  })

  const blocks: LayoutBlock[] = []
  blocks.push({
    type: 'field_group',
    label: 'Overview',
    fields: summaryKeys.length > 0 ? summaryKeys.slice(0, 3) : keys.slice(0, 3),
  })
  if (codeField) {
    blocks.push({ type: 'code', label: fieldLabel(fieldByKey, codeField), field: codeField })
  }
  if (imageField) {
    blocks.push({ type: 'image', label: fieldLabel(fieldByKey, imageField), field: imageField })
  }
  if (calloutField) {
    blocks.push({ type: 'callout', label: fieldLabel(fieldByKey, calloutField), field: calloutField, tone: 'tip' })
  }
  return blocks
}

export function inferSummaryBlocks(node: Node, fieldByKey: Map<string, NoteFieldDefinition>): LayoutBlock[] {
  const fallback = inferFallbackBlocks(node, fieldByKey)
  return fallback.map((block, index) => {
    if (index === 0 && block.type === 'field_group') {
      return {
        ...block,
        compact: true,
        fields: (block.fields ?? []).slice(0, 2),
      }
    }
    return { ...block, compact: true }
  }).slice(0, 3)
}
