<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { NoteType, NoteTypeInput } from '@/bindings'
import { useSettings } from '@/composables/useSettings'

type EditableField = {
  key: string
  label: string
  widget: string
}

type EditablePage = {
  id: string
  label: string
  kind: 'content' | 'built_in' | 'extension'
  source: string
  fieldKeys: string
}

const graphStore = useGraphStore()
const settings = useSettings()

const selectedNoteTypeId = ref<string | null>(null)
const draftName = ref('')
const draftWorldId = ref<string | null>(null)
const draftBaseNoteTypeId = ref<string | null>(null)
const draftIsDefault = ref(false)
const draftFields = ref<EditableField[]>([])
const draftPages = ref<EditablePage[]>([])
const draftMetadata = ref('{}')
const createName = ref('')

const saveStatus = ref('')

const selectedNoteType = computed(() =>
  selectedNoteTypeId.value ? graphStore.noteTypes.find(noteType => noteType.id === selectedNoteTypeId.value) ?? null : null
)

function parseJson<T>(raw: string | null | undefined, fallback: T): T {
  if (!raw) return fallback
  try {
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

function loadNoteTypeDraft(noteType: NoteType | null) {
  draftName.value = noteType?.name ?? ''
  draftWorldId.value = noteType?.world_id ?? graphStore.worldConfig?.id ?? null
  draftBaseNoteTypeId.value = noteType?.base_note_type_id ?? null
  draftIsDefault.value = noteType?.is_default ?? false
  draftMetadata.value = noteType?.metadata ?? '{}'

  const schema = parseJson<{ fields?: Array<{ key?: string; label?: string; widget?: string }> }>(
    noteType?.schema_json,
    {},
  )
  draftFields.value = (schema.fields ?? []).map(field => ({
    key: field.key ?? '',
    label: field.label ?? field.key ?? '',
    widget: field.widget ?? 'text',
  }))

  const layout = parseJson<{ pages?: Array<{ id?: string; label?: string; kind?: string; source?: string; extension_id?: string; sections?: Array<{ items?: Array<{ field?: string }> }> }> }>(
    noteType?.layout_json,
    {},
  )
  draftPages.value = (layout.pages ?? []).map(page => ({
    id: page.id ?? crypto.randomUUID(),
    label: page.label ?? page.id ?? 'Page',
    kind: page.kind === 'built_in' || page.kind === 'extension' ? page.kind : 'content',
    source: page.kind === 'extension' ? (page.extension_id ?? page.source ?? '') : (page.source ?? ''),
    fieldKeys:
      page.kind === 'content'
        ? (page.sections ?? []).flatMap(section => (section.items ?? []).map(item => item.field ?? '')).filter(Boolean).join(', ')
        : '',
  }))
}

function noteTypeToInput(): NoteTypeInput {
  const normalizedFields = draftFields.value.filter(field => field.key.trim())
  const schemaJson = JSON.stringify({
    version: 1,
    fields: normalizedFields.map(field => ({
      key: field.key.trim(),
      label: field.label.trim() || field.key.trim(),
      type: 'string',
      widget: field.widget,
    })),
  })
  const layoutJson = JSON.stringify({
    version: 1,
    pages: draftPages.value
      .filter(page => page.id.trim())
      .map(page => {
        if (page.kind === 'content') {
          return {
            id: page.id.trim(),
            label: page.label.trim() || page.id.trim(),
            kind: 'content',
            sections: [
              {
                id: `${page.id.trim()}-main`,
                label: 'Main',
                items: page.fieldKeys
                  .split(',')
                  .map(key => key.trim())
                  .filter(Boolean)
                  .map(field => ({ field })),
              },
            ],
          }
        }
        if (page.kind === 'extension') {
          return {
            id: page.id.trim(),
            label: page.label.trim() || page.id.trim(),
            kind: 'extension',
            extension_id: page.source.trim(),
          }
        }
        return {
          id: page.id.trim(),
          label: page.label.trim() || page.id.trim(),
          kind: 'built_in',
          source: page.source.trim(),
        }
      }),
  })

  return {
    name: draftName.value.trim(),
    world_id: draftWorldId.value,
    base_note_type_id: draftBaseNoteTypeId.value,
    fields: normalizedFields.map(field => field.key.trim()),
    schema_json: schemaJson,
    layout_json: layoutJson,
    metadata: draftMetadata.value.trim() || '{}',
    is_default: draftIsDefault.value,
  }
}

watch(
  () => graphStore.noteTypes,
  (noteTypes) => {
    if (!selectedNoteTypeId.value && noteTypes.length > 0) selectedNoteTypeId.value = noteTypes[0].id
    if (selectedNoteTypeId.value && !noteTypes.some(noteType => noteType.id === selectedNoteTypeId.value)) {
      selectedNoteTypeId.value = noteTypes[0]?.id ?? null
    }
  },
  { deep: true, immediate: true },
)

watch(selectedNoteType, noteType => loadNoteTypeDraft(noteType), { immediate: true })
async function saveNoteType() {
  if (!selectedNoteType.value) return
  saveStatus.value = 'Saving note type...'
  try {
    const updated = await graphStore.updateNoteType(selectedNoteType.value.id, noteTypeToInput())
    selectedNoteTypeId.value = updated.id
    saveStatus.value = 'Note type saved.'
  } catch (error) {
    saveStatus.value = String(error)
  }
}

async function createNoteType() {
  if (!createName.value.trim()) return
  saveStatus.value = 'Creating note type...'
  try {
    const created = await graphStore.createNoteType({
      name: createName.value.trim(),
      world_id: graphStore.worldConfig?.id ?? null,
      base_note_type_id: 'basic',
      fields: ['Front', 'Back'],
      schema_json: '',
      layout_json: '',
      metadata: '{}',
      is_default: false,
    })
    selectedNoteTypeId.value = created.id
    createName.value = ''
    saveStatus.value = 'Note type created.'
  } catch (error) {
    saveStatus.value = String(error)
  }
}

async function duplicateNoteType() {
  if (!selectedNoteType.value) return
  saveStatus.value = 'Duplicating note type...'
  try {
    const duplicated = await graphStore.duplicateNoteType(
      selectedNoteType.value.id,
      `${selectedNoteType.value.name} Copy`,
      graphStore.worldConfig?.id ?? null,
    )
    selectedNoteTypeId.value = duplicated.id
    saveStatus.value = 'Note type duplicated.'
  } catch (error) {
    saveStatus.value = String(error)
  }
}

function addField() {
  draftFields.value.push({ key: `field_${draftFields.value.length + 1}`, label: 'New Field', widget: 'text' })
}

function addPage() {
  draftPages.value.push({
    id: `page_${draftPages.value.length + 1}`,
    label: `Page ${draftPages.value.length + 1}`,
    kind: 'content',
    source: '',
    fieldKeys: draftFields.value.map(field => field.key).join(', '),
  })
}
</script>

<template>
  <div class="authoring-layout">
    <section class="authoring-card note-type-list">
      <div class="section-title">Note Types</div>
      <div class="create-row">
        <input v-model="createName" type="text" placeholder="New note type name" />
        <button class="authoring-btn" @click="createNoteType">Create</button>
      </div>
      <button
        v-for="noteType in graphStore.noteTypes"
        :key="noteType.id"
        class="note-type-item"
        :class="{ active: noteType.id === selectedNoteTypeId }"
        @click="selectedNoteTypeId = noteType.id"
      >
        <span>{{ noteType.name }}</span>
        <small>{{ noteType.world_id ? 'Pack' : 'Global' }}</small>
      </button>
      <button v-if="selectedNoteType" class="authoring-btn secondary" @click="duplicateNoteType">Duplicate selected</button>
    </section>

    <section class="authoring-card">
      <div class="section-title">Note Type Editor</div>
      <template v-if="selectedNoteType">
        <label class="field-block">
          <span>Name</span>
          <input v-model="draftName" type="text" />
        </label>
        <div class="grid-two">
          <label class="field-block">
            <span>Scope</span>
            <select v-model="draftWorldId">
              <option :value="null">Global</option>
              <option v-if="graphStore.worldConfig" :value="graphStore.worldConfig.id">Current pack</option>
            </select>
          </label>
          <label class="field-block">
            <span>Base note type</span>
            <select v-model="draftBaseNoteTypeId">
              <option :value="null">None</option>
              <option v-for="noteType in graphStore.noteTypes" :key="noteType.id" :value="noteType.id">{{ noteType.name }}</option>
            </select>
          </label>
        </div>
        <label class="checkbox-row">
          <input v-model="draftIsDefault" type="checkbox" />
          <span>Default for this scope</span>
        </label>

        <div class="editor-block">
          <div class="block-head">
            <strong>Fields</strong>
            <button class="authoring-btn secondary" @click="addField">Add field</button>
          </div>
          <div v-for="(field, index) in draftFields" :key="`${field.key}-${index}`" class="grid-three">
            <input v-model="field.key" type="text" placeholder="key" />
            <input v-model="field.label" type="text" placeholder="label" />
            <select v-model="field.widget">
              <option value="text">Text</option>
              <option value="long_text">Long text</option>
              <option value="markdown">Markdown</option>
              <option value="code">Code</option>
              <option value="image">Image</option>
              <option value="diagram">Diagram</option>
            </select>
          </div>
        </div>

        <div class="editor-block">
          <div class="block-head">
            <strong>Pages</strong>
            <button class="authoring-btn secondary" @click="addPage">Add page</button>
          </div>
          <div v-for="(page, index) in draftPages" :key="`${page.id}-${index}`" class="page-editor">
            <div class="grid-three">
              <input v-model="page.id" type="text" placeholder="page id" />
              <input v-model="page.label" type="text" placeholder="page label" />
              <select v-model="page.kind">
                <option value="content">Content</option>
                <option value="built_in">Built-in</option>
                <option value="extension">Extension</option>
              </select>
            </div>
            <input
              v-if="page.kind === 'content'"
              v-model="page.fieldKeys"
              type="text"
              placeholder="field keys, comma separated"
            />
            <select v-else-if="page.kind === 'built_in'" v-model="page.source">
              <option value="connections">Connections</option>
              <option value="learning">Learning</option>
              <option value="history">History</option>
            </select>
            <input v-else v-model="page.source" type="text" placeholder="extension id" />
          </div>
        </div>

        <label class="field-block">
          <span>Metadata JSON</span>
          <textarea v-model="draftMetadata" rows="4" />
        </label>

        <button class="authoring-btn" @click="saveNoteType">Save note type</button>
      </template>
      <div v-else class="empty-copy">No note type selected.</div>
    </section>

    <section class="authoring-card authoring-help">
      <div class="section-title">Node Editing</div>
      <div class="empty-copy">
        Focus a node in the graph and use the dedicated node editor instead of Settings.
      </div>
      <div class="empty-copy">
        Hotkey: <strong>{{ settings.keys.editNode.toUpperCase() }}</strong>
      </div>
      <div v-if="saveStatus" class="status-copy">{{ saveStatus }}</div>
    </section>
  </div>
</template>

<style scoped>
.authoring-layout {
  display: grid;
  grid-template-columns: 280px minmax(0, 1.2fr) minmax(240px, 0.72fr);
  gap: 12px;
}

.authoring-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.create-row,
.grid-two,
.grid-three,
.block-head,
.checkbox-row {
  display: grid;
  gap: 8px;
}

.create-row,
.grid-two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.grid-three {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.block-head {
  grid-template-columns: 1fr auto;
  align-items: center;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-block span,
.checkbox-row span,
.empty-copy,
.status-copy {
  font-size: 12px;
  color: var(--app-text-secondary);
}

.authoring-btn {
  border: 1px solid color-mix(in srgb, var(--app-accent) 40%, transparent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  border-radius: 8px;
  padding: 7px 10px;
  cursor: pointer;
}

.authoring-btn.secondary {
  background: rgba(255, 255, 255, 0.04);
  color: #d5d8e6;
  border-color: rgba(255, 255, 255, 0.12);
}

.note-type-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
  color: var(--app-text-primary);
  cursor: pointer;
}

.note-type-item.active {
  border-color: color-mix(in srgb, var(--app-accent) 40%, transparent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
}

.note-type-item small {
  color: var(--app-text-secondary);
}

.editor-block,
.page-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.authoring-help {
  justify-content: flex-start;
}

@media (max-width: 1100px) {
  .authoring-layout {
    grid-template-columns: 1fr;
  }
}
</style>
