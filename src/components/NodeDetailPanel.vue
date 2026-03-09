<script setup lang="ts">
import { X, Tag, ArrowRight, CheckCircle2, Pin, Crosshair, Clock3, History, ChevronLeft, ChevronRight, PanelsTopLeft, Shapes, Pencil, Orbit } from 'lucide-vue-next'
import { computed, ref, watch } from 'vue'
import { useEventListener } from '@vueuse/core'
import OverlayShell from '@/components/ui/OverlayShell.vue'
import NoteTypePageRenderer from '@/components/node/NoteTypePageRenderer.vue'
import NodeSummaryRenderer from '@/components/node/NodeSummaryRenderer.vue'
import NodeExtensionOutlet from '@/components/node/NodeExtensionOutlet.vue'
import { appKernel } from '@/core/kernel'

const graphStore = useGraphStore()
const settings = useSettings()

type ProgressStatus = 'new' | 'learning' | 'review' | 'mastered'
type ReviewGrade = 'again' | 'hard' | 'good' | 'easy'
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
  kind?: 'content' | 'built_in' | 'extension'
  source?: string
  slot?: string
  extension_id?: string
  sections?: LayoutSection[]
}
type ViewerPage =
  | { id: string; kind: 'content'; label: string; pageId: string }
  | { id: string; kind: 'connections'; label: string }
  | { id: string; kind: 'learning'; label: string }
  | { id: string; kind: 'history'; label: string }
  | { id: string; kind: 'extension'; label: string; extensionId: string }

const STATUS_META: Record<ProgressStatus, { label: string; className: string }> = {
  new: { label: 'New', className: 'status-new' },
  learning: { label: 'Learning', className: 'status-learning' },
  review: { label: 'Review', className: 'status-review' },
  mastered: { label: 'Mastered', className: 'status-mastered' },
}

const activePageIndex = ref(0)

const node = computed(() => graphStore.selectedNode)
const isCentered = computed(() => graphStore.centeredNodePanel)
const isPinned = computed(() => graphStore.isNodePinned(node.value?.id))
const isFocusView = computed(() => graphStore.focusViewActive)

const progressStatus = computed<ProgressStatus>(() => {
  const raw = node.value?.progress_status
  if (raw === 'learning' || raw === 'review' || raw === 'mastered') return raw
  return 'new'
})

const connectionSummary = computed(() => {
  if (!node.value) return { context: 0, prerequisite: 0, semantic: 0, custom: 0 }
  return node.value.connections.reduce(
    (acc, conn) => {
      if (conn.edge_type === 'Context') acc.context += 1
      else if (conn.edge_type === 'Prerequisite') acc.prerequisite += 1
      else if (conn.edge_type === 'Semantic') acc.semantic += 1
      else acc.custom += 1
      return acc
    },
    { context: 0, prerequisite: 0, semantic: 0, custom: 0 }
  )
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

function parseJson<T>(raw: string | null | undefined, fallback: T): T {
  if (!raw) return fallback
  try {
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

const contentPages = computed<LayoutPage[]>(() => {
  if (activeNoteType.value) {
    const parsed = parseJson<{ pages?: LayoutPage[] }>(activeNoteType.value.layout_json, {})
    if (Array.isArray(parsed.pages) && parsed.pages.length > 0) {
      return parsed.pages.filter(page => (page.kind ?? 'content') === 'content')
    }
  }

  const fallbackFields = Object.keys(node.value?.note_fields ?? {})
  const items = fallbackFields.length > 0 ? fallbackFields.map(field => ({ field })) : [{ field: 'content_data' }]

  return [
    {
      id: 'content',
      label: activeNoteType.value?.name || 'Content',
      sections: [{ id: 'main', label: 'Core', items }],
    },
  ]
})

const primaryExtensions = computed(() =>
  appKernel.listNodeWorkspaceExtensions().filter(extension => extension.slot === 'extensions.primary')
)

const explicitBuiltInPages = computed<ViewerPage[]>(() => {
  if (!activeNoteType.value) return []
  const parsed = parseJson<{ pages?: LayoutPage[] }>(activeNoteType.value.layout_json, {})
  const pages = Array.isArray(parsed.pages) ? parsed.pages : []
  const builtIns = pages.filter(page => page.kind === 'built_in')
  return builtIns
    .map((page) => {
      const label = page.label || page.id
      switch (page.source) {
        case 'connections':
          return { id: page.id, kind: 'connections' as const, label }
        case 'learning':
          return { id: page.id, kind: 'learning' as const, label }
        case 'history':
          return { id: page.id, kind: 'history' as const, label }
        default:
          return null
      }
    })
    .filter((page): page is ViewerPage => !!page)
})

const explicitExtensionPages = computed<ViewerPage[]>(() => {
  if (!activeNoteType.value) return []
  const parsed = parseJson<{ pages?: LayoutPage[] }>(activeNoteType.value.layout_json, {})
  const pages = Array.isArray(parsed.pages) ? parsed.pages : []
  const layoutPages = pages.filter(page => page.kind === 'extension')
  return layoutPages
    .map((page) => {
      const extensionId = page.extension_id || page.source
      if (!extensionId) return null
      return {
        id: page.id,
        kind: 'extension' as const,
        label: page.label || page.id,
        extensionId,
      }
    })
    .filter((page): page is ViewerPage => !!page)
})

const hasExplicitPageLayout = computed(() => {
  if (!activeNoteType.value) return false
  const parsed = parseJson<{ pages?: LayoutPage[] }>(activeNoteType.value.layout_json, {})
  return Array.isArray(parsed.pages) && parsed.pages.length > 0
})

const viewerPages = computed<ViewerPage[]>(() => [
  ...contentPages.value.map(page => ({
    id: `content:${page.id}`,
    kind: 'content' as const,
    label: page.label || page.id,
    pageId: page.id,
  })),
  ...(hasExplicitPageLayout.value
    ? explicitBuiltInPages.value
    : [
        { id: 'connections', kind: 'connections' as const, label: 'Connections' },
        { id: 'learning', kind: 'learning' as const, label: 'Learning' },
        { id: 'history', kind: 'history' as const, label: 'History' },
      ]),
  ...(hasExplicitPageLayout.value
    ? explicitExtensionPages.value
    : primaryExtensions.value.map(extension => ({
        id: `extension:${extension.id}`,
        kind: 'extension' as const,
        label: extension.title,
        extensionId: extension.id,
      }))),
])

const safePageIndex = computed(() => {
  if (viewerPages.value.length === 0) return 0
  return Math.min(activePageIndex.value, viewerPages.value.length - 1)
})

const currentPage = computed(() => viewerPages.value[safePageIndex.value] ?? null)

const nodeTips = computed(() => {
  if (!node.value) return []
  const tips: string[] = []
  if (connectionSummary.value.prerequisite > 0) tips.push('Review prerequisite links first to reduce confusion.')
  if (connectionSummary.value.context > 0) tips.push('Traverse context links to reinforce real usage patterns.')
  if (node.value.tags.length > 0) tips.push(`Use tags (${node.value.tags.slice(0, 2).join(', ')}) to group related review sessions.`)
  if (!node.value.learned) tips.push('Mark this node learned after recalling it without hints.')
  return tips.slice(0, 3)
})

const nodeHistory = computed(() =>
  graphStore.reviewEvents
    .filter(event => event.node_id === node.value?.id)
    .slice(0, 16)
)

function edgeLabel(type: string): string {
  switch (type) {
    case 'Prerequisite':
      return 'Prerequisite'
    case 'Semantic':
      return 'Related'
    case 'UserDefined':
      return 'Linked'
    case 'Context':
    default:
      return 'Context'
  }
}

function edgeBadgeClass(type: string): string {
  switch (type) {
    case 'Prerequisite':
      return 'badge-blue'
    case 'Semantic':
      return 'badge-muted'
    case 'UserDefined':
      return 'badge-amber'
    default:
      return 'badge-grey'
  }
}

function connectedNodeTitle(targetId: string): string {
  return graphStore.nodes.find(n => n.id === targetId)?.title ?? targetId.slice(0, 8)
}

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
  graphStore.selectNode(null)
}

function toggleCentered() {
  graphStore.toggleCenteredNodePanel()
  if (graphStore.centeredNodePanel) activePageIndex.value = 0
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

async function onNoteTypeChange(e: Event) {
  if (!node.value) return
  const target = e.target as HTMLSelectElement
  const next = target.value || null
  await graphStore.setNodeNoteType(node.value.id, next)
}

function cyclePage(direction: 1 | -1) {
  if (viewerPages.value.length === 0) return
  activePageIndex.value = (safePageIndex.value + direction + viewerPages.value.length) % viewerPages.value.length
}

watch(
  () => [node.value?.id, isCentered.value],
  () => {
    activePageIndex.value = 0
  }
)

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    if (!isCentered.value) return
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    if (isInput) return

    if (e.key === 'Tab') {
      e.preventDefault()
      e.stopImmediatePropagation()
      cyclePage(e.ctrlKey || e.shiftKey ? -1 : 1)
    }

    if (graphStore.focusViewActive && (e.key === 'ArrowRight' || e.key === 'ArrowLeft' || e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
      return
    }

    if (e.key === 'ArrowRight') {
      e.preventDefault()
      e.stopImmediatePropagation()
      cyclePage(1)
    }

    if (e.key === 'ArrowLeft') {
      e.preventDefault()
      e.stopImmediatePropagation()
      cyclePage(-1)
    }
  },
  { capture: true }
)
</script>

<template>
  <template v-if="node">
    <Transition name="panel">
      <div v-if="!isCentered" :key="`side-${node.id}`" class="detail-panel is-side" @click.stop>
        <div class="panel-header">
          <div class="title-wrap">
            <div class="panel-title">{{ node.title }}</div>
            <div class="panel-subtitle">{{ node.node_type }} · {{ STATUS_META[progressStatus].label }}</div>
          </div>
          <div class="header-actions">
            <button
              class="icon-btn"
              :class="{ active: isCentered }"
              @click="toggleCentered"
              :aria-label="`Open viewer (${settings.keys.openNode.toUpperCase()})`"
            >
              <Crosshair :size="13" />
            </button>
            <button
              class="icon-btn"
              :class="{ active: isPinned }"
              @click="togglePinned"
              :aria-label="`Toggle pin (${settings.keys.pinNode.toUpperCase()})`"
            >
              <Pin :size="13" />
            </button>
            <button
              class="icon-btn"
              @click="openNodeEditor"
              :aria-label="`Edit node (${settings.keys.editNode.toUpperCase()})`"
            >
              <Pencil :size="13" />
            </button>
            <button
              v-if="isFocusView"
              class="icon-btn active"
              @click="toggleFocusView"
              aria-label="Exit focus view"
            >
              <Orbit :size="13" />
            </button>
            <button class="close-btn" @click="onClose" aria-label="Close">
              <X :size="14" />
            </button>
          </div>
        </div>

        <div class="panel-divider" />

        <div class="panel-body compact-body">
          <div class="summary-chip-row">
            <span :class="['progress-chip', STATUS_META[progressStatus].className]">{{ STATUS_META[progressStatus].label }}</span>
            <span class="meta-chip">{{ node.connections.length }} links</span>
            <span class="meta-chip">{{ noteTypeName }}</span>
            <span v-if="isFocusView" class="meta-chip meta-chip-focus">Focus</span>
          </div>

          <NodeSummaryRenderer :key="`summary-${node.id}`" :node="node" :note-type="activeNoteType" />

          <template v-if="!isFocusView">
            <div class="compact-block">
              <div class="section-label">
                <Clock3 :size="12" />
                <span>Learning</span>
              </div>
              <div class="compact-fact-row">
                <span>Next review</span>
                <strong>{{ formatSchedule(node.progress_next_review_at) }}</strong>
              </div>
              <div class="compact-fact-row">
                <span>Reviews</span>
                <strong>{{ node.progress_review_count }}</strong>
              </div>
            </div>

            <div class="compact-block">
              <div class="section-label">
                <ArrowRight :size="12" />
                <span>Connections</span>
              </div>
              <ul class="connections-list compact-connections">
                <li
                  v-for="conn in node.connections.slice(0, 6)"
                  :key="conn.id"
                  class="connection-item"
                  @click="graphStore.selectNode(conn.target_id)"
                >
                  <span class="conn-target">{{ connectedNodeTitle(conn.target_id) }}</span>
                  <span :class="['conn-badge', edgeBadgeClass(conn.edge_type)]">{{ edgeLabel(conn.edge_type) }}</span>
                </li>
              </ul>
            </div>
          </template>
        </div>

        <template v-if="!isFocusView">
          <div class="panel-divider" />

          <div class="panel-footer">
            <button class="pin-btn" :class="{ active: isPinned }" @click="togglePinned">
              <Pin :size="13" />
              <span>{{ isPinned ? 'Unpin node' : 'Pin node' }} ({{ settings.keys.pinNode.toUpperCase() }})</span>
            </button>
            <button class="workspace-btn" @click="toggleCentered">
              <PanelsTopLeft :size="13" />
              <span>Open viewer ({{ settings.keys.openNode.toUpperCase() }})</span>
            </button>
            <button class="workspace-btn" @click="openNodeEditor">
              <Pencil :size="13" />
              <span>Edit node ({{ settings.keys.editNode.toUpperCase() }})</span>
            </button>
          </div>
        </template>
      </div>
    </Transition>

    <OverlayShell
      :key="`centered-${node.id}`"
      :open="isCentered"
      :title="node.title"
      :subtitle="`${node.node_type} · ${noteTypeName}`"
      width-class="node-workspace-shell"
      height-class="node-workspace-shell"
      @close="toggleCentered"
    >
      <template #actions>
        <span class="page-counter">{{ safePageIndex + 1 }} / {{ viewerPages.length }}</span>
        <span :class="['progress-chip', STATUS_META[progressStatus].className]">{{ STATUS_META[progressStatus].label }}</span>
        <button class="workspace-icon-btn" @click="openNodeEditor">
          <Pencil :size="14" />
        </button>
        <button class="workspace-icon-btn" :class="{ active: isPinned }" @click="togglePinned">
          <Pin :size="14" />
        </button>
      </template>

      <div class="viewer-layout">
        <div class="viewer-toolbar">
          <button class="viewer-nav-btn" @click="cyclePage(-1)" :disabled="viewerPages.length <= 1" aria-label="Previous page">
            <ChevronLeft :size="16" />
          </button>
          <div class="viewer-page-meta">
            <div class="viewer-page-label">{{ currentPage?.label }}</div>
            <div class="viewer-page-subtitle">Use the arrows or `Tab` to move between pages.</div>
          </div>
          <button class="viewer-nav-btn" @click="cyclePage(1)" :disabled="viewerPages.length <= 1" aria-label="Next page">
            <ChevronRight :size="16" />
          </button>
        </div>

        <div class="viewer-dots">
          <button
            v-for="(page, index) in viewerPages"
            :key="page.id"
            class="viewer-dot"
            :class="{ active: index === safePageIndex }"
            :title="page.label"
            @click="activePageIndex = index"
          />
        </div>

        <div class="viewer-stage">
          <section v-if="currentPage?.kind === 'content'" class="viewer-page viewer-page-content">
            <article class="viewer-card viewer-card-main">
              <div class="section-label">
                <Shapes :size="12" />
                <span>{{ currentPage.label }}</span>
              </div>
              <NoteTypePageRenderer :key="`page-${node.id}-${currentPage.pageId}`" :node="node" :note-type="activeNoteType" :active-page-id="currentPage.pageId" />
            </article>
          </section>

          <section v-else-if="currentPage?.kind === 'connections'" class="viewer-page">
            <div class="viewer-grid">
              <article class="viewer-card">
                <div class="section-label">
                  <Tag :size="12" />
                  <span>Structure</span>
                </div>
                <div class="facts-grid">
                  <div class="fact-cell">
                    <span>Note type</span><strong>{{ noteTypeName }}</strong>
                  </div>
                  <div class="fact-cell">
                    <span>Weight</span><strong>{{ node.weight.toFixed(2) }}</strong>
                  </div>
                  <div class="fact-cell">
                    <span>Context</span><strong>{{ connectionSummary.context }}</strong>
                  </div>
                  <div class="fact-cell">
                    <span>Prereq</span><strong>{{ connectionSummary.prerequisite }}</strong>
                  </div>
                </div>
                <div class="note-type-row">
                  <span class="note-type-label">Note type</span>
                  <select class="note-type-select" :value="node.note_type_id ?? ''" @change="onNoteTypeChange">
                    <option value="">Unassigned</option>
                    <option v-for="nt in graphStore.noteTypes" :key="nt.id" :value="nt.id">{{ nt.name }}</option>
                  </select>
                </div>
                <div v-if="node.tags.length" class="tags-block">
                  <div class="section-label">
                    <Tag :size="12" />
                    <span>Tags</span>
                  </div>
                  <div class="tags-list">
                    <span v-for="tag in node.tags" :key="tag" class="tag-badge">{{ tag }}</span>
                  </div>
                </div>
              </article>

              <article class="viewer-card">
                <div class="section-label">
                  <ArrowRight :size="12" />
                  <span>Connections</span>
                </div>
                <ul class="connections-list">
                  <li
                    v-for="conn in node.connections"
                    :key="conn.id"
                    class="connection-item"
                    @click="graphStore.selectNode(conn.target_id)"
                  >
                    <span class="conn-target">{{ connectedNodeTitle(conn.target_id) }}</span>
                    <span :class="['conn-badge', edgeBadgeClass(conn.edge_type)]">{{ edgeLabel(conn.edge_type) }}</span>
                  </li>
                </ul>
              </article>

              <article v-if="nodeTips.length" class="viewer-card viewer-card-wide">
                <div class="section-label">
                  <ArrowRight :size="12" />
                  <span>Tips</span>
                </div>
                <ul class="tips-list">
                  <li v-for="tip in nodeTips" :key="tip">{{ tip }}</li>
                </ul>
              </article>
            </div>
          </section>

          <section v-else-if="currentPage?.kind === 'learning'" class="viewer-page">
            <div class="viewer-grid">
              <article class="viewer-card viewer-card-wide">
                <div class="section-label">
                  <CheckCircle2 :size="12" />
                  <span>Learning status</span>
                </div>
                <div class="progress-head">
                  <span :class="['progress-chip', STATUS_META[progressStatus].className]">{{ STATUS_META[progressStatus].label }}</span>
                  <span class="progress-meta">{{ node.progress_review_count }} reviews · {{ node.progress_streak }} streak</span>
                </div>
                <div class="schedule-meta">
                  <span>Next review</span>
                  <strong>{{ formatSchedule(node.progress_next_review_at) }}</strong>
                </div>
                <div class="review-actions">
                  <button class="review-btn review-again" @click="reviewNode('again')">Again</button>
                  <button class="review-btn review-hard" @click="reviewNode('hard')">Hard</button>
                  <button class="review-btn review-good" @click="reviewNode('good')">Good</button>
                  <button class="review-btn review-easy" @click="reviewNode('easy')">Easy</button>
                </div>
                <div class="progress-actions">
                  <button
                    v-for="status in (['new', 'learning', 'review', 'mastered'] as ProgressStatus[])"
                    :key="status"
                    :class="['progress-btn', STATUS_META[status].className, { active: progressStatus === status }]"
                    @click="setProgressStatus(status)"
                  >
                    {{ STATUS_META[status].label }}
                  </button>
                </div>
              </article>

              <article class="viewer-card">
                <div class="section-label">
                  <Clock3 :size="12" />
                  <span>Schedule</span>
                </div>
                <div class="facts-grid">
                  <div class="fact-cell">
                    <span>Scheduler</span><strong>{{ node.progress_scheduler_key }}</strong>
                  </div>
                  <div class="fact-cell">
                    <span>Last reviewed</span><strong>{{ formatSchedule(node.progress_last_reviewed_at) }}</strong>
                  </div>
                </div>
              </article>

              <article class="viewer-card">
                <div class="section-label">
                  <CheckCircle2 :size="12" />
                  <span>Compatibility</span>
                </div>
                <button :class="['learn-btn', node.learned ? 'learned' : 'unlearned']" @click="onMarkLearned">
                  <CheckCircle2 :size="14" />
                  <span>{{ node.learned ? 'Mark as Unseen' : 'Mark as Learned' }}</span>
                </button>
              </article>

              <article class="viewer-card viewer-card-wide">
                <div class="section-label">
                  <PanelsTopLeft :size="12" />
                  <span>Extension slots</span>
                </div>
                <NodeExtensionOutlet :key="`learning-extension-${node.id}`" :node="node" slot="learning.secondary" />
              </article>
            </div>
          </section>

          <section v-else-if="currentPage?.kind === 'history'" class="viewer-page">
            <div class="viewer-grid">
              <article class="viewer-card viewer-card-wide">
                <div class="section-label">
                  <History :size="12" />
                  <span>Review history</span>
                </div>
                <div v-if="nodeHistory.length === 0" class="empty-history">
                  No review history yet for this node.
                </div>
                <div v-else class="history-list">
                  <article v-for="event in nodeHistory" :key="event.id" class="history-row">
                    <div class="history-main">
                      <strong>{{ event.grade }}</strong>
                      <span class="history-meta">{{ event.previous_status }} -> {{ event.next_status }} · {{ event.scheduler_key }}</span>
                    </div>
                    <div class="history-side">
                      <span class="history-date">{{ formatSchedule(event.reviewed_at) }}</span>
                    </div>
                  </article>
                </div>
              </article>

              <article class="viewer-card viewer-card-wide">
                <div class="section-label">
                  <PanelsTopLeft :size="12" />
                  <span>Extension slots</span>
                </div>
                <NodeExtensionOutlet :key="`history-extension-${node.id}`" :node="node" slot="history.secondary" />
              </article>
            </div>
          </section>

          <section v-else-if="currentPage?.kind === 'extension'" class="viewer-page">
            <div class="viewer-grid">
              <article class="viewer-card viewer-card-wide">
                <div class="section-label">
                  <PanelsTopLeft :size="12" />
                  <span>{{ currentPage.label }}</span>
                </div>
                <NodeExtensionOutlet :key="`extension-${node.id}-${currentPage.extensionId}`" :node="node" slot="extensions.primary" :extension-id="currentPage.extensionId" />
              </article>
            </div>
          </section>
        </div>
      </div>
    </OverlayShell>
  </template>
</template>

<style scoped>
.detail-panel {
  position: fixed;
  width: 360px;
  max-height: 78vh;
  overflow-y: auto;
  background: color-mix(in srgb, var(--app-overlay-bg) 90%, transparent);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--app-overlay-border);
  border-radius: 12px;
  color: var(--app-text-primary);
  font-family: system-ui, sans-serif;
  font-size: 13px;
  z-index: var(--z-node-detail);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.detail-panel.is-side {
  top: 50%;
  right: 20px;
  transform: translateY(-50%);
}

.panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  padding: 14px 16px 12px;
}

.panel-title {
  font-size: 15px;
  font-weight: 600;
  line-height: 1.3;
  color: var(--app-text-primary);
  flex: 1;
}

.title-wrap {
  min-width: 0;
  flex: 1;
}

.panel-subtitle {
  margin-top: 3px;
  font-size: 11px;
  color: var(--app-text-secondary);
  text-transform: capitalize;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-btn,
.close-btn,
.workspace-icon-btn {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 6px;
  color: #7a8099;
  cursor: pointer;
}

.icon-btn:hover,
.icon-btn.active,
.workspace-icon-btn:hover,
.workspace-icon-btn.active {
  background: color-mix(in srgb, var(--app-accent) 20%, transparent);
  color: var(--app-accent);
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8eaf0;
}

.panel-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
  margin: 0;
}

.panel-body {
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.compact-body {
  gap: 14px;
}

.summary-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.meta-chip {
  border-radius: 999px;
  padding: 4px 8px;
  font-size: 11px;
  color: #b8bdd0;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.meta-chip-focus {
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 26%, transparent);
}

.compact-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.compact-fact-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  font-size: 12px;
  color: #c8cad6;
}

.compact-fact-row strong {
  color: #e8eaf0;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--app-text-secondary);
  margin-bottom: 6px;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.tag-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 20px;
  background: color-mix(in srgb, var(--app-accent) 16%, transparent);
  color: var(--app-accent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 28%, transparent);
}

.connections-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 240px;
  overflow-y: auto;
  padding-right: 4px;
}

.compact-connections {
  max-height: 180px;
}

.connection-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}

.connection-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.conn-target {
  font-size: 12px;
  color: #c8cad6;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.conn-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  flex-shrink: 0;
}

.badge-blue {
  background: color-mix(in srgb, var(--app-accent) 18%, transparent);
  color: var(--app-accent);
}

.badge-muted {
  background: rgba(120, 130, 170, 0.18);
  color: #8090b0;
}

.badge-amber {
  background: rgba(245, 158, 11, 0.18);
  color: #f59e0b;
}

.badge-grey {
  background: rgba(90, 100, 140, 0.18);
  color: #6a7a9a;
}

.panel-footer {
  padding: 12px 18px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  position: sticky;
  bottom: 0;
  background: linear-gradient(180deg, rgba(12, 16, 28, 0) 0%, rgba(12, 16, 28, 0.95) 28%);
  backdrop-filter: blur(8px);
}

.focus-action-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.focus-hint {
  font-size: 11px;
  color: var(--app-text-secondary);
}


.pin-btn,
.workspace-btn,
.learn-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 12px;
  cursor: pointer;
}

.pin-btn,
.workspace-btn {
  border: 1px solid color-mix(in srgb, var(--app-accent) 38%, transparent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  color: var(--app-accent);
}

.pin-btn.active {
  background: color-mix(in srgb, var(--app-accent) 20%, transparent);
}

.learn-btn {
  border: 1px solid;
  font-size: 13px;
  font-weight: 500;
  transition: background 0.15s, border-color 0.15s;
}

.learn-btn.unlearned {
  background: rgba(61, 214, 140, 0.12);
  border-color: rgba(61, 214, 140, 0.4);
  color: #3dd68c;
}

.learn-btn.learned {
  background: rgba(120, 130, 170, 0.1);
  border-color: rgba(120, 130, 170, 0.25);
  color: #7a8099;
}

.page-counter {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.viewer-layout {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 18px;
}

.viewer-toolbar {
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr) 42px;
  gap: 12px;
  align-items: center;
}

.viewer-nav-btn {
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  border: none;
  background: rgba(255, 255, 255, 0.05);
  color: #b7c1dc;
  cursor: pointer;
}

.viewer-nav-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--app-accent) 20%, transparent);
  color: var(--app-accent);
}

.viewer-nav-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

.viewer-page-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: center;
  text-align: center;
}

.viewer-page-label {
  font-size: 18px;
  font-weight: 700;
  color: var(--app-text-primary);
}

.viewer-page-subtitle {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.viewer-dots {
  display: flex;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}

.viewer-dot {
  width: 10px;
  height: 10px;
  border: none;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
  cursor: pointer;
}

.viewer-dot.active {
  width: 28px;
  background: color-mix(in srgb, var(--app-accent) 90%, white 8%);
}

.viewer-stage {
  flex: 1;
  min-height: 0;
}

.viewer-page {
  min-height: 100%;
}

.viewer-page-content {
  display: flex;
}

.viewer-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.viewer-card {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.03));
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 18px;
  padding: 18px;
}

.viewer-card-main {
  width: min(880px, 100%);
  margin: 0 auto;
}

.viewer-card-wide {
  grid-column: 1 / -1;
}

.tags-block {
  margin-top: 16px;
}

.workspace-layout {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  min-height: 0;
  height: 100%;
}

.workspace-nav {
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.workspace-tab {
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.03);
  color: #c8cad8;
  padding: 10px 12px;
  cursor: pointer;
  text-align: left;
}

.workspace-tab.active {
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 28%, transparent);
}

.workspace-main {
  min-width: 0;
  overflow: auto;
}

.workspace-page {
  padding: 16px;
}

.workspace-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.workspace-card {
  background: rgba(255, 255, 255, 0.035);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  padding: 14px;
}

.workspace-card-wide {
  grid-column: 1 / -1;
}

.facts-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.fact-cell {
  padding: 7px 8px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.fact-cell span,
.progress-meta,
.schedule-meta span,
.history-meta,
.history-date,
.note-type-label {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.fact-cell strong,
.schedule-meta strong {
  font-size: 12px;
  color: #e8eaf0;
  font-weight: 600;
}

.note-type-row {
  margin-top: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.note-type-select {
  min-width: 180px;
}

.progress-head,
.schedule-meta,
.progress-actions,
.review-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.schedule-meta {
  flex-direction: column;
  align-items: flex-start;
}

.tips-list {
  margin: 0;
  padding-left: 16px;
  color: #c8cad6;
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 12px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border-radius: 10px;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.history-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.empty-history {
  font-size: 12px;
  color: var(--app-text-secondary);
}

.progress-chip,
.progress-btn,
.review-btn {
  border-radius: 999px;
  border: 1px solid transparent;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.progress-chip {
  padding: 4px 8px;
}

.progress-btn,
.review-btn {
  cursor: pointer;
  padding: 6px 10px;
}

.progress-btn.active {
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.18);
}

.review-again {
  color: #fda4af;
  background: rgba(244, 63, 94, 0.12);
  border-color: rgba(244, 63, 94, 0.28);
}

.review-hard {
  color: #fdba74;
  background: rgba(249, 115, 22, 0.12);
  border-color: rgba(249, 115, 22, 0.28);
}

.review-good {
  color: #93c5fd;
  background: rgba(59, 130, 246, 0.12);
  border-color: rgba(59, 130, 246, 0.28);
}

.review-easy {
  color: #86efac;
  background: rgba(34, 197, 94, 0.12);
  border-color: rgba(34, 197, 94, 0.28);
}

.status-new {
  color: #7dd3fc;
  background: rgba(14, 165, 233, 0.12);
  border-color: rgba(14, 165, 233, 0.28);
}

.status-learning {
  color: #fbbf24;
  background: rgba(245, 158, 11, 0.12);
  border-color: rgba(245, 158, 11, 0.28);
}

.status-review {
  color: #c084fc;
  background: rgba(168, 85, 247, 0.12);
  border-color: rgba(168, 85, 247, 0.28);
}

.status-mastered {
  color: #4ade80;
  background: rgba(34, 197, 94, 0.12);
  border-color: rgba(34, 197, 94, 0.28);
}

.panel-enter-active,
.panel-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.panel-enter-from,
.panel-leave-to {
  opacity: 0;
  transform: translateY(-46%);
}

@media (max-width: 980px) {
  .viewer-layout {
    padding: 14px;
  }

  .viewer-grid {
    grid-template-columns: 1fr;
  }

  .viewer-page-label {
    font-size: 16px;
  }
}
</style>
