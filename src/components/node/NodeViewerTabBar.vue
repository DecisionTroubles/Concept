<script setup lang="ts">
export type ViewerTab = {
  id: string
  label: string
}

defineProps<{
  tabs: ViewerTab[]
  activeId: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
}>()
</script>

<template>
  <div class="viewer-tabbar">
    <div class="viewer-tabbar-scroller">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="viewer-tab"
        :class="{ active: tab.id === activeId }"
        @click="emit('select', tab.id)"
      >
        {{ tab.label }}
      </button>
    </div>
    <div v-if="$slots.hint" class="viewer-tabbar-hint">
      <slot name="hint" />
    </div>
  </div>
</template>

<style scoped>
.viewer-tabbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  min-width: 0;
  padding-bottom: 0.2rem;
}

.viewer-tabbar-scroller {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0.25rem;
  overflow-x: auto;
}

.viewer-tabbar-scroller::-webkit-scrollbar {
  height: 0;
}

.viewer-tab {
  position: relative;
  border: none;
  background: transparent;
  color: var(--app-text-secondary);
  font-size: 0.9rem;
  font-weight: 600;
  padding: 0.65rem 0.8rem 0.75rem;
  white-space: nowrap;
  cursor: pointer;
}

.viewer-tab::after {
  content: '';
  position: absolute;
  left: 0.8rem;
  right: 0.8rem;
  bottom: 0.1rem;
  height: 2px;
  border-radius: 999px;
  background: transparent;
}

.viewer-tab:hover {
  color: var(--app-text-primary);
}

.viewer-tab.active {
  color: var(--app-text-primary);
}

.viewer-tab.active::after {
  background: var(--app-accent);
}

.viewer-tabbar-hint {
  flex: 0 0 auto;
  color: var(--app-text-secondary);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  white-space: nowrap;
}

@media (max-width: 820px) {
  .viewer-tabbar {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.35rem;
  }
}
</style>
