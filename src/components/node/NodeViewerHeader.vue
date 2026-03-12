<script setup lang="ts">
defineProps<{
  title: string
  subtitle: string
  noteTypeName: string
  statusLabel: string
  statusClass: string
  parentTitle?: string | null
}>()

const emit = defineEmits<{
  openParent: []
}>()
</script>

<template>
  <header class="viewer-header">
    <div class="viewer-heading">
      <h2 class="viewer-title">{{ title }}</h2>
      <div class="viewer-meta">
        <span class="viewer-note-type">{{ noteTypeName }}</span>
        <span :class="['viewer-status', statusClass]">{{ statusLabel }}</span>
        <button v-if="parentTitle" class="viewer-parent-btn" @click="emit('openParent')">
          Parent: {{ parentTitle }}
        </button>
      </div>
      <p class="viewer-subtitle">{{ subtitle }}</p>
    </div>
  </header>
</template>

<style scoped>
.viewer-header {
  display: block;
}

.viewer-heading {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}

.viewer-title {
  margin: 0;
  font-size: clamp(1.25rem, 1.5vw, 1.7rem);
  line-height: 1.1;
  color: var(--app-text-primary);
}

.viewer-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.viewer-note-type,
.viewer-status {
  font-size: 0.76rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.viewer-note-type {
  color: var(--app-text-secondary);
}

.viewer-subtitle {
  margin: 0;
  max-width: 52rem;
  color: var(--app-text-secondary);
  line-height: 1.6;
}

.viewer-parent-btn {
  border: 1px solid color-mix(in srgb, var(--app-accent) 25%, transparent);
  background: color-mix(in srgb, var(--app-accent) 8%, transparent);
  color: var(--app-text-primary);
  border-radius: 999px;
  padding: 0.35rem 0.7rem;
  cursor: pointer;
  font-size: 0.76rem;
}
</style>
