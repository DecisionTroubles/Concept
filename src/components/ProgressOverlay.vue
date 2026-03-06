<script setup lang="ts">
import { computed } from 'vue'
import { useEventListener } from '@vueuse/core'
import { Brain, Clock3, GraduationCap, Layers3 } from 'lucide-vue-next'
import OverlayShell from '@/components/ui/OverlayShell.vue'

const graphStore = useGraphStore()
const settings = useSettings()

type ProgressStatus = 'new' | 'learning' | 'review' | 'mastered'
type ReviewGrade = 'again' | 'hard' | 'good' | 'easy'

const STATUS_ORDER: ProgressStatus[] = ['new', 'learning', 'review', 'mastered']

const statusMeta: Record<ProgressStatus, { label: string; tone: string; hint: string }> = {
  new: {
    label: 'New',
    tone: 'status-new',
    hint: 'Not started yet',
  },
  learning: {
    label: 'Learning',
    tone: 'status-learning',
    hint: 'Currently being acquired',
  },
  review: {
    label: 'Review',
    tone: 'status-review',
    hint: 'Needs repeated exposure',
  },
  mastered: {
    label: 'Mastered',
    tone: 'status-mastered',
    hint: 'Stable and retained',
  },
}

const progressCounts = computed(() => {
  const counts: Record<ProgressStatus, number> = {
    new: 0,
    learning: 0,
    review: 0,
    mastered: 0,
  }

  for (const node of graphStore.nodes) {
    const key = (node.progress_status in counts ? node.progress_status : 'new') as ProgressStatus
    counts[key] += 1
  }

  return counts
})

const totalNodes = computed(() => graphStore.nodes.length)
const completionRatio = computed(() => {
  if (totalNodes.value === 0) return 0
  return progressCounts.value.mastered / totalNodes.value
})

const recentReviewEvents = computed(() =>
  graphStore.reviewEvents.slice(0, 16).map(event => ({
    ...event,
    nodeTitle: graphStore.nodes.find(node => node.id === event.node_id)?.title ?? event.node_id,
  }))
)

const nextUpNodes = computed(() =>
  [...graphStore.nodes]
    .filter(node => node.progress_status !== 'mastered')
    .sort((a, b) => {
      const aRank = STATUS_ORDER.indexOf((a.progress_status in statusMeta ? a.progress_status : 'new') as ProgressStatus)
      const bRank = STATUS_ORDER.indexOf((b.progress_status in statusMeta ? b.progress_status : 'new') as ProgressStatus)
      if (aRank !== bRank) return aRank - bRank
      return a.title.localeCompare(b.title)
    })
    .slice(0, 12)
)

function displayStatus(status: string): string {
  return statusMeta[(status in statusMeta ? status : 'new') as ProgressStatus].label
}

function focusNode(nodeId: string) {
  graphStore.requestFocus(nodeId)
}

async function setStatus(nodeId: string, status: ProgressStatus) {
  await graphStore.setNodeProgressStatus(nodeId, status)
}

async function reviewNode(nodeId: string, grade: ReviewGrade) {
  await graphStore.reviewNode(nodeId, grade)
}

function formatSchedule(ts: string | null): string {
  if (!ts) return 'No schedule yet'
  const numeric = Number(ts)
  if (!Number.isFinite(numeric) || numeric <= 0) return 'No schedule yet'
  return new Date(numeric * 1000).toLocaleString()
}

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    if (isInput) return

    const key = e.key.toLowerCase()
    if (key === settings.keys.progressOverlay) {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.toggleProgressOverlay()
      return
    }

    if (graphStore.progressOverlayOpen && e.key === 'Escape') {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.closeProgressOverlay()
    }
  },
  { capture: true }
)
</script>

<template>
  <OverlayShell
    :open="graphStore.progressOverlayOpen"
    title="Learning Progress"
    subtitle="Dedicated overview for current state, due work, and node-level updates"
    width-class="progress-shell"
    height-class="progress-shell"
    @close="graphStore.closeProgressOverlay()"
  >
    <div class="progress-layout">
      <section class="progress-summary">
        <div class="summary-head">
          <div>
            <div class="eyebrow">Overview</div>
            <h2>Current world progress</h2>
          </div>
          <div class="summary-pill">
            <GraduationCap :size="14" />
            <span>{{ Math.round(completionRatio * 100) }}% mastered</span>
          </div>
        </div>

        <div class="progress-bar">
          <div class="progress-bar-fill" :style="{ width: `${completionRatio * 100}%` }" />
        </div>

        <div class="summary-grid">
          <article v-for="status in STATUS_ORDER" :key="status" class="summary-card" :class="statusMeta[status].tone">
            <div class="summary-card-head">
              <span>{{ statusMeta[status].label }}</span>
              <strong>{{ progressCounts[status] }}</strong>
            </div>
            <p>{{ statusMeta[status].hint }}</p>
          </article>
        </div>
      </section>

      <section class="progress-side">
        <article class="insight-card">
          <div class="insight-head">
            <Brain :size="15" />
            <span>Due now</span>
          </div>
          <strong>{{ graphStore.dueNodes.length }}</strong>
          <p>Nodes with no schedule yet or `next_review_at` already due.</p>
        </article>

        <article class="insight-card">
          <div class="insight-head">
            <Layers3 :size="15" />
            <span>Active layer</span>
          </div>
          <strong>{{ graphStore.layers.find(l => l.id === graphStore.activeLayerId)?.name ?? 'None' }}</strong>
          <p>Progress is computed from the nodes currently loaded into the 3D world.</p>
        </article>

        <article class="insight-card">
          <div class="insight-head">
            <Clock3 :size="15" />
            <span>Hotkey</span>
          </div>
          <strong>{{ settings.keys.progressOverlay.toUpperCase() }}</strong>
          <p>Opens this progress window directly. It is separate from `B` buffers.</p>
        </article>

        <article class="insight-card">
          <div class="insight-head">
            <Clock3 :size="15" />
            <span>Scheduler</span>
          </div>
          <strong>
            {{ graphStore.schedulerAlgorithms.find(x => x.key === settings.learning.defaultSchedulerKey)?.name ?? settings.learning.defaultSchedulerKey }}
          </strong>
          <p>Default scheduler from Settings. Review actions use it unless a specific override is passed.</p>
        </article>
      </section>

      <section class="progress-list">
        <div class="list-head">
          <div>
            <div class="eyebrow">Queue</div>
            <h3>Nodes to work through next</h3>
          </div>
          <div class="queue-meta">{{ nextUpNodes.length }} shown / {{ graphStore.dueNodes.length }} due</div>
        </div>

        <div v-if="nextUpNodes.length === 0" class="empty-state">
          Everything in the current world is marked mastered.
        </div>

        <div v-else class="queue-list">
          <article v-for="node in nextUpNodes" :key="node.id" class="queue-card">
            <div class="queue-card-head">
              <button class="node-link" @click="focusNode(node.id)">{{ node.title }}</button>
              <span class="status-chip" :class="statusMeta[(node.progress_status in statusMeta ? node.progress_status : 'new') as ProgressStatus].tone">
                {{ displayStatus(node.progress_status) }}
              </span>
            </div>

            <p class="queue-copy">{{ node.content_data || 'No note content yet.' }}</p>

            <div class="queue-meta-row">
              <span>{{ node.node_type }}</span>
              <span>{{ node.tags.slice(0, 3).join(' · ') || 'untagged' }}</span>
              <span>{{ node.connections.length }} links</span>
            </div>

            <div class="schedule-row">
              <span>Next review</span>
              <strong>{{ formatSchedule(node.progress_next_review_at) }}</strong>
            </div>

            <div class="review-actions">
              <button class="grade-btn grade-again" @click="reviewNode(node.id, 'again')">Again</button>
              <button class="grade-btn grade-hard" @click="reviewNode(node.id, 'hard')">Hard</button>
              <button class="grade-btn grade-good" @click="reviewNode(node.id, 'good')">Good</button>
              <button class="grade-btn grade-easy" @click="reviewNode(node.id, 'easy')">Easy</button>
            </div>

            <div class="queue-actions">
              <button class="status-btn status-new" @click="setStatus(node.id, 'new')">New</button>
              <button class="status-btn status-learning" @click="setStatus(node.id, 'learning')">Learning</button>
              <button class="status-btn status-review" @click="setStatus(node.id, 'review')">Review</button>
              <button class="status-btn status-mastered" @click="setStatus(node.id, 'mastered')">Mastered</button>
            </div>
          </article>
        </div>
      </section>

      <section class="progress-list">
        <div class="list-head">
          <div>
            <div class="eyebrow">History</div>
            <h3>Recent review events</h3>
          </div>
          <div class="queue-meta">{{ recentReviewEvents.length }} events</div>
        </div>

        <div v-if="recentReviewEvents.length === 0" class="empty-state">
          No review history yet. Use the review buttons to start generating events.
        </div>

        <div v-else class="history-list">
          <article v-for="event in recentReviewEvents" :key="event.id" class="history-row">
            <div class="history-main">
              <button class="node-link" @click="focusNode(event.node_id)">{{ event.nodeTitle }}</button>
              <span class="history-meta">
                {{ event.previous_status }} → {{ event.next_status }} · {{ event.scheduler_key }}
              </span>
            </div>
            <div class="history-side">
              <span class="grade-chip" :class="`grade-${event.grade}`">{{ event.grade }}</span>
              <span class="history-date">{{ formatSchedule(event.reviewed_at) }}</span>
            </div>
          </article>
        </div>
      </section>
    </div>
  </OverlayShell>
</template>

<style scoped>
.progress-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.5fr) minmax(280px, 0.72fr);
  gap: 16px;
  padding: 16px;
}

.progress-summary,
.progress-list,
.progress-side {
  min-width: 0;
}

.progress-summary,
.progress-list,
.insight-card {
  background: rgba(255, 255, 255, 0.035);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
}

.progress-summary,
.progress-list {
  padding: 16px;
}

.progress-side {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.summary-head,
.list-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.eyebrow {
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

h2,
h3 {
  margin: 4px 0 0;
  font-size: 22px;
  color: var(--app-text-primary);
}

.summary-pill,
.queue-meta {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--app-accent);
  border-radius: 999px;
  padding: 7px 10px;
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 28%, transparent);
  white-space: nowrap;
}

.progress-bar {
  margin-top: 16px;
  height: 12px;
  border-radius: 999px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.08);
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #3dd68c, color-mix(in srgb, var(--app-accent) 80%, #ffffff 20%));
}

.summary-grid {
  margin-top: 16px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.summary-card {
  border-radius: 12px;
  padding: 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
}

.summary-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  color: var(--app-text-primary);
}

.summary-card-head strong {
  font-size: 20px;
}

.summary-card p {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--app-text-secondary);
  line-height: 1.4;
}

.insight-card {
  padding: 14px;
}

.insight-head {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.insight-card strong {
  display: block;
  margin-top: 12px;
  font-size: 22px;
  color: var(--app-text-primary);
}

.insight-card p {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--app-text-secondary);
  line-height: 1.45;
}

.progress-list {
  grid-column: 1 / -1;
}

.empty-state {
  margin-top: 14px;
  padding: 16px;
  border-radius: 12px;
  color: var(--app-text-secondary);
  background: rgba(255, 255, 255, 0.03);
}

.queue-list {
  margin-top: 16px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}

.history-list {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  border-radius: 12px;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.history-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.history-meta,
.history-date {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.history-side {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.queue-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-radius: 12px;
  padding: 14px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.02));
  border: 1px solid rgba(255, 255, 255, 0.09);
}

.queue-card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.node-link {
  border: none;
  background: none;
  padding: 0;
  color: var(--app-text-primary);
  font-size: 15px;
  font-weight: 700;
  text-align: left;
  cursor: pointer;
}

.queue-copy {
  margin: 0;
  color: #c8cad8;
  font-size: 12px;
  line-height: 1.5;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
}

.queue-meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 11px;
  color: var(--app-text-secondary);
}

.queue-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.review-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.schedule-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.schedule-row span {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.schedule-row strong {
  font-size: 12px;
  color: var(--app-text-primary);
}

.status-chip,
.status-btn,
.grade-btn {
  border-radius: 999px;
  border: 1px solid transparent;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.status-chip {
  padding: 4px 8px;
  white-space: nowrap;
}

.status-btn {
  cursor: pointer;
  padding: 6px 10px;
  background: rgba(255, 255, 255, 0.04);
}

.grade-btn {
  cursor: pointer;
  padding: 6px 10px;
}

.grade-again {
  color: #fda4af;
  background: rgba(244, 63, 94, 0.12);
  border-color: rgba(244, 63, 94, 0.28);
}

.grade-hard {
  color: #fdba74;
  background: rgba(249, 115, 22, 0.12);
  border-color: rgba(249, 115, 22, 0.28);
}

.grade-good {
  color: #93c5fd;
  background: rgba(59, 130, 246, 0.12);
  border-color: rgba(59, 130, 246, 0.28);
}

.grade-easy {
  color: #86efac;
  background: rgba(34, 197, 94, 0.12);
  border-color: rgba(34, 197, 94, 0.28);
}

.grade-chip {
  border-radius: 999px;
  border: 1px solid transparent;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  padding: 5px 9px;
  text-transform: capitalize;
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

@media (max-width: 920px) {
  .progress-layout {
    grid-template-columns: 1fr;
  }
}
</style>
