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

const OVERVIEW_PRIMARY_KEYS = ['summary', 'meaning', 'function', 'main', 'concept', 'term', 'phrase', 'pattern']
const OVERVIEW_SUPPORT_KEYS = ['why', 'whyitmatters', 'usage', 'when']
const SIGNAL_KEYS = ['signals', 'mapsignals', 'tip']
const PITFALL_KEYS = ['pitfall', 'warning', 'caution', 'note']
const EXAMPLE_KEYS = ['example', 'examplecode', 'diagram', 'visual']

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

function normalized(value: string): string {
  return value.toLowerCase().replaceAll(/[^a-z0-9]/g, '')
}

function fieldKeys(node: Node): string[] {
  return Object.keys(node.note_fields)
}

function firstMatchingKey(keys: string[], needles: string[]): string | null {
  for (const key of keys) {
    const norm = normalized(key)
    if (needles.some(needle => norm.includes(needle))) return key
  }
  return null
}

function allMatchingKeys(keys: string[], needles: string[]): string[] {
  return keys.filter(key => {
    const norm = normalized(key)
    return needles.some(needle => norm.includes(needle))
  })
}

function fieldWidget(fieldByKey: Map<string, NoteFieldDefinition>, key: string): string {
  return (fieldByKey.get(key)?.widget || '').toLowerCase()
}

function preferCodeField(keys: string[], fieldByKey: Map<string, NoteFieldDefinition>): string | null {
  const explicitCode = keys.find(key => fieldWidget(fieldByKey, key) === 'code')
  if (explicitCode) return explicitCode
  return firstMatchingKey(keys, ['examplecode', 'code', 'snippet'])
}

function preferImageField(keys: string[], fieldByKey: Map<string, NoteFieldDefinition>): string | null {
  const explicitImage = keys.find(key => {
    const widget = fieldWidget(fieldByKey, key)
    return widget === 'image' || widget === 'diagram'
  })
  if (explicitImage) return explicitImage
  return firstMatchingKey(keys, ['diagram', 'image', 'visual'])
}

function candidateOverviewFields(keys: string[]): string[] {
  const ordered = [
    ...allMatchingKeys(keys, OVERVIEW_PRIMARY_KEYS),
    ...allMatchingKeys(keys, OVERVIEW_SUPPORT_KEYS),
  ]
  return Array.from(new Set(ordered))
}

function candidateExampleFields(keys: string[], fieldByKey: Map<string, NoteFieldDefinition>): string[] {
  const ordered = [
    ...(preferCodeField(keys, fieldByKey) ? [preferCodeField(keys, fieldByKey)!] : []),
    ...allMatchingKeys(keys, EXAMPLE_KEYS),
  ]
  return Array.from(new Set(ordered))
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

export function inferFallbackContentPages(node: Node, fieldByKey: Map<string, NoteFieldDefinition>): LayoutPage[] {
  const keys = fieldKeys(node)
  const overviewFields = candidateOverviewFields(keys)
  const whyField = firstMatchingKey(keys, OVERVIEW_SUPPORT_KEYS)
  const signalField = firstMatchingKey(keys, SIGNAL_KEYS)
  const pitfallField = firstMatchingKey(keys, PITFALL_KEYS)
  const codeField = preferCodeField(keys, fieldByKey)
  const exampleField = firstMatchingKey(keys, ['example'])
  const imageField = preferImageField(keys, fieldByKey)
  const exampleSupportField = firstMatchingKey(keys, ['usage', 'when', 'tip', 'function'])

  const overviewBlocks: LayoutBlock[] = []
  const primaryOverviewFields = overviewFields.length > 0
    ? overviewFields.slice(0, 2)
    : keys.slice(0, 2)

  if (primaryOverviewFields.length > 0) {
    overviewBlocks.push({
      type: 'field_group',
      label: 'Core idea',
      fields: primaryOverviewFields,
    })
  } else {
    overviewBlocks.push({
      type: 'field_group',
      label: 'Core idea',
      fields: ['content_data'],
    })
  }

  if (whyField && !primaryOverviewFields.includes(whyField)) {
    overviewBlocks.push({
      type: 'markdown',
      label: 'Why it matters',
      field: whyField,
    })
  }
  if (signalField) {
    overviewBlocks.push({
      type: 'callout',
      label: 'Map signals',
      field: signalField,
      tone: 'info',
    })
  }
  if (pitfallField) {
    overviewBlocks.push({
      type: 'callout',
      label: 'Common pitfall',
      field: pitfallField,
      tone: 'warning',
    })
  }
  overviewBlocks.push({
    type: 'relations',
    label: 'Related jumps',
    compact: true,
  })

  const exampleBlocks: LayoutBlock[] = []
  if (codeField) {
    exampleBlocks.push({
      type: 'code',
      label: fieldLabel(fieldByKey, codeField),
      field: codeField,
      language: inferCodeLanguage(node, codeField),
    })
  } else if (exampleField) {
    exampleBlocks.push({
      type: 'markdown',
      label: fieldLabel(fieldByKey, exampleField),
      field: exampleField,
    })
  } else if (imageField) {
    exampleBlocks.push({
      type: 'image',
      label: fieldLabel(fieldByKey, imageField),
      field: imageField,
      caption_field: firstMatchingKey(keys, ['caption']),
    })
  } else if (keys.length > 0) {
    exampleBlocks.push({
      type: 'field_group',
      label: 'Example',
      fields: keys.slice(0, 1),
    })
  }

  if (exampleSupportField && (!codeField || exampleSupportField !== codeField)) {
    exampleBlocks.push({
      type: 'callout',
      label: 'What to notice',
      field: exampleSupportField,
      tone: 'tip',
    })
  }
  exampleBlocks.push({
    type: 'relations',
    label: 'Related jumps',
    compact: true,
  })

  return [
    {
      id: 'overview',
      label: 'Overview',
      kind: 'content',
      blocks: overviewBlocks,
    },
    {
      id: 'example',
      label: 'Example',
      kind: 'content',
      blocks: exampleBlocks,
    },
  ]
}

export function inferFallbackBlocks(node: Node, fieldByKey: Map<string, NoteFieldDefinition>): LayoutBlock[] {
  return inferFallbackContentPages(node, fieldByKey)[0]?.blocks ?? [{ type: 'field_group', label: 'Content', fields: ['content_data'] }]
}

export function inferSummaryBlocks(node: Node, fieldByKey: Map<string, NoteFieldDefinition>): LayoutBlock[] {
  return (inferFallbackContentPages(node, fieldByKey)[0]?.blocks ?? [])
    .filter(block => block.type !== 'relations')
    .map((block, index) => {
      if (index === 0 && block.type === 'field_group') {
        return {
          ...block,
          compact: true,
          fields: (block.fields ?? []).slice(0, 2),
        }
      }
      return { ...block, compact: true }
    })
    .slice(0, 2)
}

function inferCodeLanguage(node: Node, key: string): string {
  const lowerKey = normalized(key)
  if (lowerKey.includes('odin')) return 'odin'
  if (node.title.toLowerCase().includes('odin')) return 'odin'
  return 'text'
}
