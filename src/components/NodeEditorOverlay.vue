<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useEventListener } from '@vueuse/core'
import OverlayShell from '@/components/ui/OverlayShell.vue'
import NoteTypePageRenderer from '@/components/node/NoteTypePageRenderer.vue'

const graphStore = useGraphStore()
const settings = useSettings()
const editorMode = useEditorMode()

type EditableFieldDef = { key?: string; label?: string; widget?: string }

const activeTab = ref<'content' | 'preview'>('content')
const nodeTitle = ref('')
const nodeTags = ref('')
const nodeContent = ref('')
const nodeFields = ref<Record<string, string>>({})
const saveStatus = ref('')
const newConnectionLayerName = ref('')

const node = computed(() => graphStore.selectedNode)
const authoringMode = computed(() => editorMode.mode.value === 'author')
const noteType = computed(() =>
  node.value?.note_type_id ? graphStore.noteTypes.find(note => note.id === node.value?.note_type_id) ?? null : null
)

function parseJson<T>(raw: string | null | undefined, fallback: T): T {
  if (!raw) return fallback
  try {
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

const schemaFields = computed<EditableFieldDef[]>(() =>
  parseJson<{ fields?: EditableFieldDef[] }>(noteType.value?.schema_json, {}).fields ?? []
)

function syncDraft() {
  nodeTitle.value = node.value?.title ?? ''
  nodeTags.value = (node.value?.tags ?? []).join(', ')
  nodeContent.value = node.value?.content_data ?? ''
  nodeFields.value = { ...(node.value?.note_fields ?? {}) }
  saveStatus.value = ''
}

watch(node, syncDraft, { immediate: true })

async function saveNode() {
  if (!node.value) return
  saveStatus.value = 'Saving node...'
  try {
    await graphStore.updateNodeContent(
      node.value.id,
      nodeTitle.value.trim() || node.value.title,
      nodeFields.value,
      nodeContent.value.trim() || null,
      nodeTags.value
        .split(',')
        .map(tag => tag.trim())
        .filter(Boolean),
    )
    saveStatus.value = 'Node saved.'
  } catch (error) {
    saveStatus.value = String(error)
  }
}

async function updateNodeNoteType(event: Event) {
  if (!node.value) return
  const next = (event.target as HTMLSelectElement).value || null
  await graphStore.setNodeNoteType(node.value.id, next)
}

function patchNodePosition(dx: number, dy: number, dz: number) {
  if (!node.value) return
  const x = (node.value.pos_x ?? 0) + dx
  const y = (node.value.pos_y ?? 0) + dy
  const z = (node.value.pos_z ?? 0) + dz
  node.value.pos_x = x
  node.value.pos_y = y
  node.value.pos_z = z
  graphStore.updateNodePosition(node.value.id, x, y, z)
}

function spreadNearby() {
  if (!node.value) return
  const sx = node.value.pos_x ?? 0
  const sy = node.value.pos_y ?? 0
  const sz = node.value.pos_z ?? 0
  for (const item of graphStore.nodes) {
    if (item.id === node.value.id) continue
    const dx = (item.pos_x ?? 0) - sx
    const dy = (item.pos_y ?? 0) - sy
    const dz = (item.pos_z ?? 0) - sz
    const distance = Math.sqrt(dx * dx + dy * dy + dz * dz)
    if (distance < 0.001 || distance > 10) continue
    const push = 2.5 * (1 - distance / 10)
    item.pos_x = (item.pos_x ?? 0) + (dx / distance) * push
    item.pos_y = (item.pos_y ?? 0) + (dy / distance) * push
    item.pos_z = (item.pos_z ?? 0) + (dz / distance) * push
    graphStore.updateNodePosition(item.id, item.pos_x, item.pos_y, item.pos_z)
  }
}

function expandNeighbors() {
  if (!node.value) return
  const neighborIds = new Set(node.value.connections.map(edge => edge.target_id))
  for (const item of graphStore.nodes.filter(entry => neighborIds.has(entry.id))) {
    const dx = (item.pos_x ?? 0) - (node.value.pos_x ?? 0)
    const dy = (item.pos_y ?? 0) - (node.value.pos_y ?? 0)
    const dz = (item.pos_z ?? 0) - (node.value.pos_z ?? 0)
    const distance = Math.sqrt(dx * dx + dy * dy + dz * dz)
    if (distance < 0.001) continue
    item.pos_x = (item.pos_x ?? 0) + (dx / distance) * 2
    item.pos_y = (item.pos_y ?? 0) + (dy / distance) * 2
    item.pos_z = (item.pos_z ?? 0) + (dz / distance) * 2
    graphStore.updateNodePosition(item.id, item.pos_x, item.pos_y, item.pos_z)
  }
}

async function createConnectionLayer() {
  const name = newConnectionLayerName.value.trim()
  if (!name) return
  const created = await graphStore.createConnectionLayer(
    null,
    name,
    graphStore.connectionLayers.length,
    JSON.stringify({ color: '#7dd3fc', width: 2.2, line_style: 'solid' })
  )
  graphStore.setConnectionLayerSelection([created.id])
  newConnectionLayerName.value = ''
}

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    if (!isInput && e.key.toLowerCase() === settings.keys.editNode) {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.toggleNodeEditor()
      return
    }

    if (graphStore.nodeEditorOpen && e.key === 'Escape') {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.closeNodeEditor()
    }
  },
  { capture: true }
)
</script>

<template>
  <OverlayShell
    :open="graphStore.nodeEditorOpen"
    :title="node ? `Edit ${node.title}` : 'Node Editor'"
    :subtitle="node ? `Focused node editor · ${settings.keys.editNode.toUpperCase()} to toggle` : 'Select a node to edit it'"
    width-class="node-editor-shell"
    height-class="node-editor-shell"
    @close="graphStore.closeNodeEditor()"
  >
    <div v-if="node" class="node-editor-layout">
      <div class="node-editor-tabs">
        <button class="node-editor-tab" :class="{ active: activeTab === 'content' }" @click="activeTab = 'content'">
          Content
        </button>
        <button class="node-editor-tab" :class="{ active: activeTab === 'preview' }" @click="activeTab = 'preview'">
          Preview
        </button>
      </div>

      <div v-if="activeTab === 'content'" class="node-editor-grid">
        <section class="node-editor-card">
          <div class="node-editor-section-title">Identity</div>
          <label class="node-editor-field">
            <span>Title</span>
            <input v-model="nodeTitle" type="text" />
          </label>
          <label class="node-editor-field">
            <span>Tags</span>
            <input v-model="nodeTags" type="text" placeholder="comma, separated, tags" />
          </label>
          <label class="node-editor-field">
            <span>Note type</span>
            <select :value="node.note_type_id ?? ''" @change="updateNodeNoteType">
              <option value="">Unassigned</option>
              <option v-for="item in graphStore.noteTypes" :key="item.id" :value="item.id">{{ item.name }}</option>
            </select>
          </label>
          <label class="node-editor-field">
            <span>Fallback content</span>
            <textarea v-model="nodeContent" rows="5" />
          </label>
        </section>

        <section class="node-editor-card">
          <div class="node-editor-section-title">Structured Fields</div>
          <div v-if="schemaFields.length === 0" class="node-editor-empty">
            This node type has no explicit schema fields yet.
          </div>
          <label
            v-for="field in schemaFields"
            :key="field.key"
            class="node-editor-field"
          >
            <span>{{ field.label || field.key }}</span>
            <textarea
              v-if="field.widget === 'long_text' || field.widget === 'markdown' || field.widget === 'code'"
              v-model="nodeFields[field.key || '']"
              rows="5"
            />
            <input v-else v-model="nodeFields[field.key || '']" type="text" />
          </label>
        </section>

        <section v-if="authoringMode" class="node-editor-card">
          <div class="node-editor-section-title">Authoring</div>
          <div class="node-editor-mini-grid">
            <button class="node-editor-btn subtle" @click="patchNodePosition(-1, 0, 0)">Left</button>
            <button class="node-editor-btn subtle" @click="patchNodePosition(1, 0, 0)">Right</button>
            <button class="node-editor-btn subtle" @click="patchNodePosition(0, 0, -1)">Forward</button>
            <button class="node-editor-btn subtle" @click="patchNodePosition(0, 0, 1)">Back</button>
            <button class="node-editor-btn subtle" @click="patchNodePosition(0, 1, 0)">Raise</button>
            <button class="node-editor-btn subtle" @click="patchNodePosition(0, -1, 0)">Lower</button>
            <button class="node-editor-btn subtle" @click="spreadNearby">Spread nearby</button>
            <button class="node-editor-btn subtle" @click="expandNeighbors">Expand neighbors</button>
          </div>
          <label class="node-editor-field">
            <span>Active connection layer</span>
            <select
              :value="graphStore.activeConnectionLayerIds[0] ?? ''"
              @change="graphStore.setConnectionLayerSelection([($event.target as HTMLSelectElement).value])"
            >
              <option v-for="layer in graphStore.connectionLayers" :key="layer.id" :value="layer.id">
                {{ layer.name }}
              </option>
            </select>
          </label>
          <label class="node-editor-field">
            <span>Add connection layer</span>
            <div class="node-editor-inline">
              <input v-model="newConnectionLayerName" type="text" placeholder="Memory links" />
              <button class="node-editor-btn" @click="createConnectionLayer">Add</button>
            </div>
          </label>
        </section>
      </div>

      <div v-else class="node-editor-preview">
        <section class="node-editor-card">
          <div class="node-editor-section-title">Live Viewer Preview</div>
          <NoteTypePageRenderer :node="node" :note-type="noteType" />
        </section>
      </div>

      <div class="node-editor-footer">
        <div class="node-editor-status">{{ saveStatus || 'Focused node editor separate from Settings.' }}</div>
        <div class="node-editor-actions">
          <button class="node-editor-btn subtle" @click="graphStore.closeNodeEditor()">Close</button>
          <button class="node-editor-btn" @click="saveNode">Save node</button>
        </div>
      </div>
    </div>

    <div v-else class="node-editor-empty-page">
      Select a node in the graph, then open the node editor.
    </div>
  </OverlayShell>
</template>

<style scoped>
.node-editor-layout {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
}

.node-editor-tabs {
  display: flex;
  gap: 8px;
}

.node-editor-tab,
.node-editor-btn {
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--app-accent) 30%, transparent);
  background: color-mix(in srgb, var(--app-accent) 10%, transparent);
  color: var(--app-accent);
  padding: 8px 12px;
  cursor: pointer;
}

.node-editor-tab.active {
  background: color-mix(in srgb, var(--app-accent) 18%, transparent);
}

.node-editor-grid {
  display: grid;
  grid-template-columns: minmax(0, 0.85fr) minmax(0, 1.15fr);
  gap: 14px;
}

.node-editor-preview {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.node-editor-mini-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.node-editor-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.035);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.node-editor-section-title {
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.node-editor-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.node-editor-inline {
  display: flex;
  gap: 8px;
}

.node-editor-field span,
.node-editor-status,
.node-editor-empty,
.node-editor-empty-page {
  font-size: 12px;
  color: var(--app-text-secondary);
}

.node-editor-footer {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.node-editor-actions {
  display: flex;
  gap: 8px;
}

.node-editor-btn.subtle {
  color: var(--app-text-secondary);
  border-color: rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.04);
}

.node-editor-empty-page {
  padding: 24px;
}

@media (max-width: 960px) {
  .node-editor-grid {
    grid-template-columns: 1fr;
  }

  .node-editor-footer {
    flex-direction: column;
    align-items: stretch;
  }

  .node-editor-actions {
    justify-content: flex-end;
  }
}
</style>
