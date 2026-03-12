<script setup lang="ts">
import { X, Pin, Pencil, PanelsTopLeft, Orbit, BookOpen, History as HistoryIcon, Sparkles } from 'lucide-vue-next'
import { computed, ref, watch } from 'vue'
import { useEventListener } from '@vueuse/core'
import OverlayShell from '@/components/ui/OverlayShell.vue'
import NoteTypePageRenderer from '@/components/node/NoteTypePageRenderer.vue'
import NodeSummaryRenderer from '@/components/node/NodeSummaryRenderer.vue'
import NodeExtensionOutlet from '@/components/node/NodeExtensionOutlet.vue'
import NodeViewerHeader from '@/components/node/NodeViewerHeader.vue'
import NodeViewerTabBar, { type ViewerTab } from '@/components/node/NodeViewerTabBar.vue'
import NodeConnectionsPage from '@/components/node/NodeConnectionsPage.vue'
import NodeLearningPage from '@/components/node/NodeLearningPage.vue'
import NodeHistoryPage from '@/components/node/NodeHistoryPage.vue'
import { appKernel } from '@/core/kernel'
import { inferFallbackContentPages, parseLayout, type LayoutPage } from '@/components/node/layout'

const graphStore = useGraphStore()
const settings = useSettings()

type ProgressStatus = 'new' | 'learning' | 'review' | 'mastered'
type ReviewGrade = 'again' | 'hard' | 'good' | 'easy'
type ViewerPage =
  | { id: string; kind: 'content'; label: string; pageId: string; category: 'primary' }
  | { id: 'connections'; kind: 'connections'; label: string; category: 'primary' }
  | { id: 'learning'; kind: 'learning'; label: string; category: 'secondary' }
  | { id: 'history'; kind: 'history'; label: string; category: 'secondary' }
  | { id: string; kind: 'extension'; label: string; extensionId: string; category: 'secondary' }

const STATUS_META: Record<ProgressStatus, { label: string; className: string }> = {
  new: { label: 'New', className: 'status-new' },
  learning: { label: 'Learning', className: 'status-learning' },
  review: { label: 'Review', className: 'status-review' },
  mastered: { label: 'Mastered', className: 'status-mastered' },
}

const activePageId = ref<string>('overview')

const node = computed(() => {
  const selectedId = graphStore.selectedNodeId
  if (!selectedId) return null
  return graphStore.nodes.find(candidate => candidate.id === selectedId) ?? null
})
const isCentered = computed(() => graphStore.centeredNodePanel)
const isPinned = computed(() => graphStore.isNodePinned(node.value?.id))
const isFocusView = computed(() => graphStore.focusViewActive)

const progressStatus = computed<ProgressStatus>(() => {
  const raw = node.value?.progress_status
  if (raw === 'learning' || raw === 'review' || raw === 'mastered') return raw
  return 'new'
})

const noteTypeName = computed(() => {
  const id = node.value?.note_type_id
  if (!id) return 'Unassigned'
  return graphStore.noteTypes.find(n => n.id === id)?.name ?? 'Unknown'
})

const activeNoteType = computed(() => {
  const id = node.value?.note_type_id
  if (!id) return null
  return graphStore.noteTypes.find(n => n.id === id) ?? null
})

const parentNode = computed(() => {
  const parentId = node.value?.parent_node_id
  if (!parentId) return null
  return graphStore.nodes.find(candidate => candidate.id === parentId) ?? null
})

function parseJson<T>(raw: string | null | undefined, fallback: T): T {
  if (!raw) return fallback
  try {
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

const contentPages = computed<LayoutPage[]>(() => {
  if (!node.value) return []
  const parsed = parseLayout(activeNoteType.value)
  const authored = Array.isArray(parsed.pages) ? parsed.pages.filter(page => (page.kind ?? 'content') === 'content') : []
  if (authored.length > 0) return authored

  const fieldMap = new Map(
    parseJson<{ fields?: Array<{ key: string; label?: string; widget?: string }> }>(activeNoteType.value?.schema_json, {}).fields?.map(field => [field.key, field]) ?? []
  )
  return inferFallbackContentPages(node.value, fieldMap)
})

const explicitBuiltIns = computed(() => {
  const parsed = parseLayout(activeNoteType.value)
  const pages = Array.isArray(parsed.pages) ? parsed.pages : []
  return pages.filter(page => page.kind === 'built_in')
})

const primaryPages = computed<ViewerPage[]>(() => {
  const content = contentPages.value.map(page => ({
    id: `content:${page.id}`,
    kind: 'content' as const,
    label: page.label || page.id,
    pageId: page.id,
    category: 'primary' as const,
  }))

  const hasExplicitConnections = explicitBuiltIns.value.some(page => page.source === 'connections')
  if (hasExplicitConnections || content.length === 0) {
    content.push({
      id: 'connections',
      kind: 'connections',
      label: explicitBuiltIns.value.find(page => page.source === 'connections')?.label || 'Connections',
      category: 'primary',
    })
    return content
  }

  content.push({ id: 'connections', kind: 'connections', label: 'Connections', category: 'primary' })
  return content
})

const extensionPages = computed<ViewerPage[]>(() => {
  const parsed = parseLayout(activeNoteType.value)
  const pages = Array.isArray(parsed.pages) ? parsed.pages : []
  const authored = pages
    .filter(page => page.kind === 'extension')
    .map((page) => {
      const extensionId = page.extension_id || page.source
      if (!extensionId) return null
      return {
        id: `extension:${extensionId}`,
        kind: 'extension' as const,
        label: page.label || extensionId,
        extensionId,
        category: 'secondary' as const,
      }
    })
    .filter((page): page is ViewerPage => !!page)

  if (authored.length > 0) return authored
  return appKernel.listNodeWorkspaceExtensions()
    .filter(extension => extension.slot === 'extensions.primary')
    .map(extension => ({
      id: `extension:${extension.id}`,
      kind: 'extension' as const,
      label: extension.title,
      extensionId: extension.id,
      category: 'secondary' as const,
    }))
})

const secondaryPages = computed<ViewerPage[]>(() => [
  { id: 'learning', kind: 'learning', label: 'Learning', category: 'secondary' },
  { id: 'history', kind: 'history', label: 'History', category: 'secondary' },
  ...extensionPages.value,
])

const allPages = computed<ViewerPage[]>(() => [...primaryPages.value, ...secondaryPages.value])

const currentPage = computed(() => {
  return allPages.value.find(page => page.id === activePageId.value) ?? primaryPages.value[0] ?? null
})

const primaryTabs = computed<ViewerTab[]>(() => primaryPages.value.map(page => ({ id: page.id, label: page.label })))
const utilityTabs = computed(() =>
  secondaryPages.value.map((page, index) => ({
    ...page,
    hotkey: index < 9 ? `Alt+${index + 1}` : null,
  }))
)

const nodeHistory = computed(() =>
  graphStore.reviewEvents
    .filter(event => event.node_id === node.value?.id)
    .slice(0, 20)
)

const connectionBuckets = computed(() => {
  if (!node.value) return { next: [], related: [], supporting: [] } as Record<string, Array<{ id: string; title: string; edgeType: string; relationLabel: string; targetId: string }>>

  const relationKindsById = new Map(graphStore.relationKinds.map(kind => [kind.id, kind.label]))
  const decorated = node.value.connections.map(conn => ({
    id: conn.id,
    title: graphStore.nodes.find(candidate => candidate.id === conn.target_id)?.title ?? conn.target_id.slice(0, 8),
    edgeType: conn.edge_type,
    relationLabel: relationKindsById.get(conn.relation_id ?? '') ?? conn.edge_type,
    targetId: conn.target_id,
  }))

  return {
    next: decorated.filter(conn => conn.edgeType === 'Prerequisite' || conn.edgeType === 'Context'),
    related: decorated.filter(conn => conn.edgeType === 'Semantic'),
    supporting: decorated.filter(conn => conn.edgeType === 'UserDefined'),
  }
})

const overviewExcerpt = computed(() => {
  if (!node.value) return ''
  const fields = node.value.note_fields
  return fields.Summary
    || fields.Meaning
    || fields.Function
    || fields.Main
    || fields.Concept
    || fields.Example
    || node.value.content_data
    || ''
})

const centeredSubtitle = computed(() => {
  if (!node.value) return ''
  if (parentNode.value) return `Attached to ${parentNode.value.title}. Open the core idea first, then move into examples or links.`
  return 'Read the idea first, then move into examples and graph connections.'
})

function formatSchedule(ts: string | null): string {
  if (!ts) return 'No schedule yet'
  const numeric = Number(ts)
  if (!Number.isFinite(numeric) || numeric <= 0) return 'No schedule yet'
  return new Date(numeric * 1000).toLocaleString()
}

async function onMarkLearned() {
  if (!node.value) return
  await graphStore.markLearned(node.value.id, !node.value.learned)
}

async function setProgressStatus(status: ProgressStatus) {
  if (!node.value) return
  await graphStore.setNodeProgressStatus(node.value.id, status)
}

async function reviewNode(grade: ReviewGrade) {
  if (!node.value) return
  await graphStore.reviewNode(node.value.id, grade)
}

function onClose() {
  graphStore.clearSelection()
}

function toggleCentered() {
  graphStore.toggleCenteredNodePanel()
  if (graphStore.centeredNodePanel) activePageId.value = primaryPages.value[0]?.id ?? 'overview'
}

function togglePinned() {
  if (!node.value) return
  graphStore.togglePinNode(node.value.id)
}

function openNodeEditor() {
  graphStore.openNodeEditor()
}

function toggleFocusView() {
  if (!node.value) return
  graphStore.toggleFocusView(node.value.id)
}

function openParentNode() {
  if (!parentNode.value) return
  graphStore.selectNode(parentNode.value.id)
}

function openNode(targetId: string) {
  graphStore.selectNode(targetId)
}

watch(
  () => node.value?.id,
  () => {
    activePageId.value = primaryPages.value[0]?.id ?? 'overview'
  },
  { immediate: true }
)

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    if (!isCentered.value) return
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    if (isInput) return

    const primary = primaryPages.value
    if (!primary.length) return
    const activePrimaryIndex = primary.findIndex(page => page.id === currentPage.value?.id)

    if (e.key === 'Tab' && activePrimaryIndex !== -1) {
      e.preventDefault()
      const delta = e.shiftKey ? -1 : 1
      activePageId.value = primary[(activePrimaryIndex + delta + primary.length) % primary.length]?.id ?? primary[0].id
    }

    if (e.altKey) {
      const index = Number.parseInt(e.key, 10) - 1
      if (Number.isInteger(index) && index >= 0 && index < utilityTabs.value.length) {
        e.preventDefault()
        activePageId.value = utilityTabs.value[index]?.id ?? activePageId.value
      }
    }

    if ((e.key === 'ArrowLeft' || e.key === 'ArrowRight') && currentPage.value?.category === 'secondary' && utilityTabs.value.length) {
      e.preventDefault()
      const currentUtilityIndex = utilityTabs.value.findIndex(page => page.id === currentPage.value?.id)
      if (currentUtilityIndex !== -1) {
        const delta = e.key === 'ArrowRight' ? 1 : -1
        activePageId.value = utilityTabs.value[(currentUtilityIndex + delta + utilityTabs.value.length) % utilityTabs.value.length]?.id ?? activePageId.value
      }
    }
  },
  { capture: true }
)
</script>

<template>
  <template v-if="node">
    <Transition name="panel">
      <aside v-if="!isCentered" :key="`side-${node.id}`" class="detail-panel is-side" @click.stop>
        <div class="side-head">
          <div class="side-title-wrap">
            <div class="side-title">{{ node.title }}</div>
            <div class="side-subtitle">{{ noteTypeName }}</div>
          </div>
          <div class="side-actions">
            <button class="side-icon-btn" :class="{ active: isPinned }" @click="togglePinned" :title="`Pin (${settings.keys.pinNode.toUpperCase()})`">
              <Pin :size="13" />
            </button>
            <button class="side-icon-btn" @click="openNodeEditor" :title="`Edit (${settings.keys.editNode.toUpperCase()})`">
              <Pencil :size="13" />
            </button>
            <button class="side-icon-btn" @click="toggleCentered" :title="`Open viewer (${settings.keys.openNode.toUpperCase()})`">
              <PanelsTopLeft :size="13" />
            </button>
            <button class="side-icon-btn" @click="onClose" title="Close">
              <X :size="13" />
            </button>
          </div>
        </div>

        <div class="side-meta">
          <span :class="['progress-chip', STATUS_META[progressStatus].className]">{{ STATUS_META[progressStatus].label }}</span>
          <button v-if="parentNode" class="side-parent-btn" @click="openParentNode">Parent: {{ parentNode.title }}</button>
          <span v-if="isFocusView" class="side-chip side-chip-focus">Focus</span>
          <span class="side-chip">{{ node.connections.length }} links</span>
        </div>

        <p v-if="overviewExcerpt" class="side-excerpt">{{ overviewExcerpt }}</p>

        <div class="side-summary">
          <NodeSummaryRenderer :key="`summary-${node.id}`" :node="node" :note-type="activeNoteType" />
        </div>

        <div class="side-footer">
          <div class="side-fact">
            <span>Next review</span>
            <strong>{{ formatSchedule(node.progress_next_review_at) }}</strong>
          </div>
          <button v-if="isFocusView" class="side-focus-btn" @click="toggleFocusView">
            Exit focus
            <Orbit :size="13" />
          </button>
        </div>
      </aside>
    </Transition>

    <OverlayShell
      :key="`centered-${node.id}`"
      :open="isCentered"
      width-class="node-workspace-shell"
      height-class="node-workspace-shell"
      @close="toggleCentered"
    >
      <template #title>
        <div class="viewer-topbar-meta">
          <span class="viewer-topbar-note-type">{{ noteTypeName }}</span>
          <span :class="['viewer-topbar-status', STATUS_META[progressStatus].className]">{{ STATUS_META[progressStatus].label }}</span>
          <button v-if="parentNode" class="viewer-topbar-parent" @click="openParentNode">
            Parent: {{ parentNode.title }}
          </button>
        </div>
      </template>

      <template #actions>
        <div class="viewer-topbar-actions">
          <div v-if="utilityTabs.length" class="viewer-topbar-tools">
            <button
              v-for="tool in utilityTabs"
              :key="tool.id"
              class="viewer-topbar-tool-btn"
              :class="{ active: currentPage?.id === tool.id }"
              :title="tool.hotkey ? `${tool.label} (${tool.hotkey})` : tool.label"
              @click="activePageId = tool.id"
            >
              <BookOpen v-if="tool.id === 'learning'" :size="13" />
              <HistoryIcon v-else-if="tool.id === 'history'" :size="13" />
              <Sparkles v-else :size="13" />
              <span class="viewer-topbar-tool-key">{{ tool.hotkey?.replace('Alt+', '') ?? '•' }}</span>
            </button>
          </div>

          <div class="viewer-topbar-divider" />

          <button class="viewer-topbar-action-btn" :class="{ active: isPinned }" @click="togglePinned" :title="`Pin (${settings.keys.pinNode.toUpperCase()})`">
            <Pin :size="14" />
          </button>
          <button class="viewer-topbar-action-btn" @click="openNodeEditor" :title="`Edit (${settings.keys.editNode.toUpperCase()})`">
            <Pencil :size="14" />
          </button>
          <button class="viewer-topbar-action-btn" @click="toggleCentered" title="Close detail">
            <X :size="14" />
          </button>
        </div>
      </template>

      <div class="viewer-shell">
        <NodeViewerHeader
          :title="node.title"
          :subtitle="centeredSubtitle"
          :note-type-name="noteTypeName"
          :status-label="STATUS_META[progressStatus].label"
          :status-class="STATUS_META[progressStatus].className"
          :parent-title="parentNode?.title ?? null"
          @open-parent="openParentNode"
        />

        <div class="viewer-nav">
          <NodeViewerTabBar :tabs="primaryTabs" :active-id="currentPage?.category === 'primary' ? currentPage.id : null" @select="activePageId = $event" />
        </div>

        <div class="viewer-body">
          <section v-if="currentPage?.kind === 'content'" class="viewer-reading">
            <NoteTypePageRenderer
              :key="`page-${node.id}-${currentPage.pageId}`"
              :node="node"
              :note-type="activeNoteType"
              :active-page-id="currentPage.pageId"
            />
          </section>

          <section v-else-if="currentPage?.kind === 'connections'" class="viewer-reading">
            <NodeConnectionsPage
              :note-type-name="noteTypeName"
              :status-label="STATUS_META[progressStatus].label"
              :next-review-label="formatSchedule(node.progress_next_review_at)"
              :next="connectionBuckets.next"
              :related="connectionBuckets.related"
              :supporting="connectionBuckets.supporting"
              @open-node="openNode"
            />
          </section>

          <section v-else-if="currentPage?.kind === 'learning'" class="viewer-reading viewer-reading-narrow">
            <NodeLearningPage
              :status-label="STATUS_META[progressStatus].label"
              :status-class="STATUS_META[progressStatus].className"
              :review-count="node.progress_review_count"
              :streak="node.progress_streak"
              :next-review-label="formatSchedule(node.progress_next_review_at)"
              :scheduler-key="node.progress_scheduler_key"
              :last-reviewed-label="formatSchedule(node.progress_last_reviewed_at)"
              :learned="node.learned"
              @review="reviewNode"
              @set-status="setProgressStatus"
              @toggle-learned="onMarkLearned"
            />
          </section>

          <section v-else-if="currentPage?.kind === 'history'" class="viewer-reading viewer-reading-narrow">
            <NodeHistoryPage :events="nodeHistory" :format-schedule="formatSchedule" />
          </section>

          <section v-else-if="currentPage?.kind === 'extension'" class="viewer-reading viewer-reading-narrow">
            <NodeExtensionOutlet
              :key="`extension-${node.id}-${currentPage.extensionId}`"
              :node="node"
              slot="extensions.primary"
              :extension-id="currentPage.extensionId"
            />
          </section>
        </div>
      </div>
    </OverlayShell>
  </template>
</template>

<style scoped>
.detail-panel {
  position: fixed;
  top: 50%;
  right: 20px;
  transform: translateY(-50%);
  width: min(360px, calc(100vw - 32px));
  max-height: 78vh;
  overflow-y: auto;
  z-index: var(--z-node-detail);
  border-radius: 1.05rem;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 84%, transparent);
  background: color-mix(in srgb, var(--app-overlay-bg) 92%, transparent);
  backdrop-filter: blur(18px);
  box-shadow: 0 10px 32px rgba(0, 0, 0, 0.35);
  padding: 1rem;
}

.side-head,
.side-footer {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}

.side-title-wrap {
  min-width: 0;
}

.side-title {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--app-text-primary);
}

.side-subtitle {
  margin-top: 0.2rem;
  color: var(--app-text-secondary);
  font-size: 0.85rem;
}

.side-actions {
  display: flex;
  align-items: center;
  gap: 0.35rem;
}

.side-icon-btn {
  width: 1.9rem;
  height: 1.9rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.7rem;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 78%, transparent);
  background: transparent;
  color: var(--app-text-secondary);
  cursor: pointer;
}

.side-icon-btn:hover,
.side-icon-btn.active {
  color: var(--app-accent);
  border-color: color-mix(in srgb, var(--app-accent) 28%, transparent);
  background: color-mix(in srgb, var(--app-accent) 10%, transparent);
}

.side-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin: 0.9rem 0 0.8rem;
}

.side-chip,
.progress-chip,
.side-parent-btn {
  border-radius: 999px;
  padding: 0.32rem 0.7rem;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.side-chip {
  color: var(--app-text-secondary);
  background: color-mix(in srgb, var(--app-overlay-bg) 70%, white 3%);
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 72%, transparent);
}

.side-chip-focus {
  color: var(--app-accent);
}

.side-parent-btn {
  border: 1px solid color-mix(in srgb, var(--app-accent) 28%, transparent);
  background: color-mix(in srgb, var(--app-accent) 8%, transparent);
  color: var(--app-text-primary);
  cursor: pointer;
}

.status-new {
  color: #6ab7ff;
  background: rgba(106, 183, 255, 0.12);
}

.status-learning {
  color: #ffb84f;
  background: rgba(255, 184, 79, 0.12);
}

.status-review {
  color: #a6e36f;
  background: rgba(166, 227, 111, 0.12);
}

.status-mastered {
  color: #68d6a8;
  background: rgba(104, 214, 168, 0.12);
}

.side-excerpt {
  margin: 0 0 1rem;
  color: var(--app-text-primary);
  line-height: 1.65;
}

.side-summary {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.side-footer {
  margin-top: 1rem;
  padding-top: 0.9rem;
  border-top: 1px solid color-mix(in srgb, var(--app-overlay-border) 70%, transparent);
}

.side-fact {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.side-fact span {
  color: var(--app-text-secondary);
  font-size: 0.76rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.side-fact strong {
  color: var(--app-text-primary);
  font-size: 0.9rem;
}

.side-focus-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  border: none;
  background: transparent;
  color: var(--app-accent);
  cursor: pointer;
}

.viewer-shell {
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
  min-height: 100%;
  padding: 1.25rem 1.5rem 1.5rem;
}

.viewer-nav {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  border-bottom: 1px solid color-mix(in srgb, var(--app-overlay-border) 72%, transparent);
}

.viewer-topbar-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.45rem;
  min-width: 0;
}

.viewer-topbar-note-type,
.viewer-topbar-status {
  color: var(--app-text-secondary);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.viewer-topbar-parent {
  border: 1px solid color-mix(in srgb, var(--app-accent) 25%, transparent);
  background: color-mix(in srgb, var(--app-accent) 8%, transparent);
  color: var(--app-text-primary);
  border-radius: 999px;
  padding: 0.28rem 0.62rem;
  cursor: pointer;
  font-size: 0.72rem;
}

.viewer-topbar-actions {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.viewer-topbar-tools {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  overflow-x: auto;
}

.viewer-topbar-tools::-webkit-scrollbar {
  height: 0;
}

.viewer-topbar-tool-btn,
.viewer-topbar-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 2.05rem;
  height: 2.05rem;
  border-radius: 0.78rem;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 78%, transparent);
  background: transparent;
  color: var(--app-text-secondary);
  cursor: pointer;
}

.viewer-topbar-tool-btn.active,
.viewer-topbar-tool-btn:hover,
.viewer-topbar-action-btn.active,
.viewer-topbar-action-btn:hover {
  color: var(--app-text-primary);
  border-color: color-mix(in srgb, var(--app-accent) 28%, transparent);
  background: color-mix(in srgb, var(--app-accent) 10%, transparent);
}

.viewer-topbar-tool-key {
  position: absolute;
  right: 0.22rem;
  bottom: 0.14rem;
  font-size: 0.53rem;
  font-weight: 700;
  color: color-mix(in srgb, var(--app-text-secondary) 88%, transparent);
}

.viewer-topbar-divider {
  width: 1px;
  height: 1.7rem;
  background: color-mix(in srgb, var(--app-overlay-border) 72%, transparent);
}

.viewer-body {
  flex: 1;
  min-height: 0;
}

.viewer-reading {
  width: min(920px, 100%);
  margin: 0 auto;
}

.viewer-reading-narrow {
  width: min(760px, 100%);
}

:deep(.node-workspace-shell) {
  width: min(1180px, calc(100vw - 40px));
  height: min(88vh, 920px);
}

@media (max-width: 860px) {
  .viewer-shell {
    padding: 1rem;
  }

  .viewer-nav {
    flex-direction: column;
    align-items: stretch;
  }

  .viewer-topbar-actions {
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .viewer-topbar-meta {
    gap: 0.35rem;
  }
}
</style>
