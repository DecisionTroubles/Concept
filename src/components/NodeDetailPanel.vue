<script setup lang="ts">
import { X, BookOpen, Tag, ArrowRight, CheckCircle2 } from 'lucide-vue-next'
import { computed } from 'vue'

const graphStore = useGraphStore()

const node = computed(() => graphStore.selectedNode)

// Edge type → human readable
function edgeLabel(type: string): string {
  switch (type) {
    case 'Prerequisite': return 'Prerequisite'
    case 'Semantic':     return 'Related'
    case 'UserDefined':  return 'Linked'
    case 'Context':
    default:             return 'Context'
  }
}

function edgeBadgeClass(type: string): string {
  switch (type) {
    case 'Prerequisite': return 'badge-blue'
    case 'Semantic':     return 'badge-muted'
    case 'UserDefined':  return 'badge-amber'
    default:             return 'badge-grey'
  }
}

// Find connected node titles within the store
function connectedNodeTitle(targetId: string): string {
  return graphStore.nodes.find(n => n.id === targetId)?.title ?? targetId.slice(0, 8)
}

async function onMarkLearned() {
  if (!node.value) return
  await graphStore.markLearned(node.value.id, !node.value.learned)
}

function onClose() {
  graphStore.selectNode(null)
}

</script>

<template>
  <Transition name="panel">
    <div v-if="node" class="detail-panel" @click.stop>
      <!-- Header -->
      <div class="panel-header">
        <div class="panel-title">{{ node.title }}</div>
        <button class="close-btn" @click="onClose" aria-label="Close">
          <X :size="14" />
        </button>
      </div>

      <div class="panel-divider" />

      <!-- Content -->
      <div class="panel-body">
        <div v-if="node.content_data" class="content-section">
          <div class="section-label">
            <BookOpen :size="12" />
            <span>Content</span>
          </div>
          <p class="content-text">{{ node.content_data }}</p>
        </div>

        <!-- Tags -->
        <div v-if="node.tags?.length" class="tags-section">
          <div class="section-label">
            <Tag :size="12" />
            <span>Tags</span>
          </div>
          <div class="tags-list">
            <span v-for="tag in node.tags" :key="tag" class="tag-badge">{{ tag }}</span>
          </div>
        </div>

        <!-- Connections -->
        <div v-if="node.connections?.length" class="connections-section">
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
              <span :class="['conn-badge', edgeBadgeClass(conn.edge_type)]">
                {{ edgeLabel(conn.edge_type) }}
              </span>
            </li>
          </ul>
        </div>
      </div>

      <div class="panel-divider" />

      <!-- Footer action -->
      <div class="panel-footer">
        <button
          :class="['learn-btn', node.learned ? 'learned' : 'unlearned']"
          @click="onMarkLearned"
        >
          <CheckCircle2 :size="14" />
          <span>{{ node.learned ? 'Mark as Unseen' : 'Mark as Learned' }}</span>
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.detail-panel {
  position: fixed;
  top: 50%;
  right: 20px;
  transform: translateY(-50%);
  width: 260px;
  max-height: 70vh;
  overflow-y: auto;
  background: rgba(12, 16, 28, 0.82);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  color: #e8eaf0;
  font-family: system-ui, sans-serif;
  font-size: 13px;
  z-index: 100;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
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
  color: #ffffff;
  flex: 1;
}

.close-btn {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 5px;
  color: #7a8099;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
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
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #7a8099;
  margin-bottom: 6px;
}

.content-text {
  font-size: 13px;
  line-height: 1.55;
  color: #c8cad6;
  margin: 0;
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
  background: rgba(91, 143, 255, 0.15);
  color: #5b8fff;
  border: 1px solid rgba(91, 143, 255, 0.25);
}

.connections-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
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

.badge-blue   { background: rgba(91, 143, 255, 0.18); color: #5b8fff; }
.badge-muted  { background: rgba(120, 130, 170, 0.18); color: #8090b0; }
.badge-amber  { background: rgba(245, 158, 11, 0.18); color: #f59e0b; }
.badge-grey   { background: rgba(90, 100, 140, 0.18); color: #6a7a9a; }

.panel-footer {
  padding: 12px 16px;
}

.learn-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
}

.learn-btn.unlearned {
  background: rgba(61, 214, 140, 0.12);
  border-color: rgba(61, 214, 140, 0.4);
  color: #3dd68c;
}

.learn-btn.unlearned:hover {
  background: rgba(61, 214, 140, 0.22);
}

.learn-btn.learned {
  background: rgba(120, 130, 170, 0.1);
  border-color: rgba(120, 130, 170, 0.25);
  color: #7a8099;
}

.learn-btn.learned:hover {
  background: rgba(120, 130, 170, 0.18);
}

/* Transition */
.panel-enter-active,
.panel-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.panel-enter-from,
.panel-leave-to {
  opacity: 0;
  transform: translateY(-50%) translateX(12px);
}
</style>
