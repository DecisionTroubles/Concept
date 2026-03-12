<script setup lang="ts">
defineProps<{
  statusLabel: string
  statusClass: string
  reviewCount: number
  streak: number
  nextReviewLabel: string
  schedulerKey: string
  lastReviewedLabel: string
  learned: boolean
}>()

const emit = defineEmits<{
  review: [grade: 'again' | 'hard' | 'good' | 'easy']
  setStatus: [status: 'new' | 'learning' | 'review' | 'mastered']
  toggleLearned: []
}>()
</script>

<template>
  <div class="learning-page">
    <section class="learning-hero">
      <span :class="['progress-chip', statusClass]">{{ statusLabel }}</span>
      <p>{{ reviewCount }} reviews · {{ streak }} streak · Next review {{ nextReviewLabel }}</p>
    </section>

    <section class="learning-section">
      <div class="eyebrow">Review</div>
      <div class="review-actions">
        <button class="review-btn" @click="emit('review', 'again')">Again</button>
        <button class="review-btn" @click="emit('review', 'hard')">Hard</button>
        <button class="review-btn" @click="emit('review', 'good')">Good</button>
        <button class="review-btn" @click="emit('review', 'easy')">Easy</button>
      </div>
    </section>

    <section class="learning-section">
      <div class="eyebrow">Status</div>
      <div class="status-actions">
        <button class="status-btn" @click="emit('setStatus', 'new')">New</button>
        <button class="status-btn" @click="emit('setStatus', 'learning')">Learning</button>
        <button class="status-btn" @click="emit('setStatus', 'review')">Review</button>
        <button class="status-btn" @click="emit('setStatus', 'mastered')">Mastered</button>
      </div>
    </section>

    <section class="learning-section learning-meta">
      <article>
        <span class="eyebrow">Scheduler</span>
        <strong>{{ schedulerKey }}</strong>
      </article>
      <article>
        <span class="eyebrow">Last reviewed</span>
        <strong>{{ lastReviewedLabel }}</strong>
      </article>
      <button class="compat-btn" @click="emit('toggleLearned')">
        {{ learned ? 'Mark as unseen' : 'Mark as learned' }}
      </button>
    </section>
  </div>
</template>

<style scoped>
.learning-page {
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

.learning-hero,
.learning-section,
.learning-meta {
  border-radius: 1rem;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 72%, transparent);
  background: color-mix(in srgb, var(--app-overlay-bg) 80%, white 2%);
  padding: 1rem 1.05rem;
}

.learning-hero p,
.learning-meta strong {
  margin: 0.55rem 0 0;
  color: var(--app-text-primary);
}

.eyebrow {
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.review-actions,
.status-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  margin-top: 0.75rem;
}

.review-btn,
.status-btn,
.compat-btn {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 82%, transparent);
  background: transparent;
  color: var(--app-text-primary);
  padding: 0.55rem 0.95rem;
  cursor: pointer;
}

.review-btn:hover,
.status-btn:hover,
.compat-btn:hover {
  border-color: color-mix(in srgb, var(--app-accent) 28%, transparent);
  background: color-mix(in srgb, var(--app-accent) 8%, transparent);
}

.learning-meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1rem;
  align-items: end;
}

@media (max-width: 860px) {
  .learning-meta {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
