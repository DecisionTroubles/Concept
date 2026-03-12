<script setup lang="ts">
type ConnectionItem = {
  id: string
  title: string
  edgeType: string
  relationLabel: string
  targetId: string
}

defineProps<{
  noteTypeName: string
  statusLabel: string
  nextReviewLabel: string
  next: ConnectionItem[]
  related: ConnectionItem[]
  supporting: ConnectionItem[]
}>()

const emit = defineEmits<{
  openNode: [id: string]
}>()
</script>

<template>
  <div class="connections-page">
    <section class="connection-section connection-hero">
      <div class="eyebrow">At a glance</div>
      <div class="glance-grid">
        <article class="glance-card">
          <span>Status</span>
          <strong>{{ statusLabel }}</strong>
        </article>
        <article class="glance-card">
          <span>Note type</span>
          <strong>{{ noteTypeName }}</strong>
        </article>
        <article class="glance-card">
          <span>Next review</span>
          <strong>{{ nextReviewLabel }}</strong>
        </article>
      </div>
    </section>

    <section class="connection-section">
      <div class="eyebrow">Best next hop</div>
      <div v-if="next.length" class="link-list">
        <button v-for="item in next" :key="item.id" class="link-row" @click="emit('openNode', item.targetId)">
          <span>{{ item.title }}</span>
          <small>{{ item.relationLabel }}</small>
        </button>
      </div>
      <p v-else class="empty-copy">No progression links from this node yet.</p>
    </section>

    <section class="connection-section two-up">
      <article class="cluster-panel">
        <div class="eyebrow">Related ideas</div>
        <div v-if="related.length" class="link-list">
          <button v-for="item in related" :key="item.id" class="link-row" @click="emit('openNode', item.targetId)">
            <span>{{ item.title }}</span>
            <small>{{ item.relationLabel }}</small>
          </button>
        </div>
        <p v-else class="empty-copy">No conceptual links yet.</p>
      </article>

      <article class="cluster-panel">
        <div class="eyebrow">Supporting links</div>
        <div v-if="supporting.length" class="link-list">
          <button v-for="item in supporting" :key="item.id" class="link-row" @click="emit('openNode', item.targetId)">
            <span>{{ item.title }}</span>
            <small>{{ item.relationLabel }}</small>
          </button>
        </div>
        <p v-else class="empty-copy">No supporting links yet.</p>
      </article>
    </section>
  </div>
</template>

<style scoped>
.connections-page {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.connection-section {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.connection-hero {
  padding-bottom: 0.35rem;
  border-bottom: 1px solid color-mix(in srgb, var(--app-overlay-border) 74%, transparent);
}

.glance-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.8rem;
}

.glance-card,
.cluster-panel {
  padding: 1rem 1.05rem;
  border-radius: 1rem;
  background: color-mix(in srgb, var(--app-overlay-bg) 80%, white 2%);
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 72%, transparent);
}

.glance-card span,
.eyebrow,
.link-row small {
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.glance-card strong {
  display: block;
  margin-top: 0.45rem;
  color: var(--app-text-primary);
}

.link-list {
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
}

.link-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  width: 100%;
  text-align: left;
  padding: 0.9rem 1rem;
  border-radius: 0.9rem;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 72%, transparent);
  background: transparent;
  color: var(--app-text-primary);
  cursor: pointer;
}

.link-row:hover {
  border-color: color-mix(in srgb, var(--app-accent) 26%, transparent);
  background: color-mix(in srgb, var(--app-accent) 7%, transparent);
}

.empty-copy {
  margin: 0;
  color: var(--app-text-secondary);
}

.two-up {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}

@media (max-width: 860px) {
  .glance-grid,
  .two-up {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
