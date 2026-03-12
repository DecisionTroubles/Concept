<script setup lang="ts">
import type { ReviewEvent } from '@/bindings'

defineProps<{
  events: ReviewEvent[]
  formatSchedule: (value: string | null) => string
}>()
</script>

<template>
  <div class="history-page">
    <div v-if="events.length === 0" class="history-empty">No review history yet for this node.</div>
    <article v-for="event in events" :key="event.id" class="history-row">
      <div class="history-main">
        <strong>{{ event.grade }}</strong>
        <span>{{ event.previous_status }} -> {{ event.next_status }}</span>
      </div>
      <div class="history-side">
        <span>{{ event.scheduler_key }}</span>
        <time>{{ formatSchedule(event.reviewed_at) }}</time>
      </div>
    </article>
  </div>
</template>

<style scoped>
.history-page {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.history-empty {
  color: var(--app-text-secondary);
}

.history-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.95rem 1rem;
  border-radius: 0.95rem;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 72%, transparent);
  background: color-mix(in srgb, var(--app-overlay-bg) 80%, white 2%);
}

.history-main,
.history-side {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.history-main strong {
  color: var(--app-text-primary);
  text-transform: capitalize;
}

.history-main span,
.history-side {
  color: var(--app-text-secondary);
  font-size: 0.9rem;
}
</style>
