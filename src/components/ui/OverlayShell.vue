<script setup lang="ts">
defineProps<{
  open: boolean
  title?: string
  subtitle?: string
  widthClass?: string
  heightClass?: string
}>()

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <Teleport to="body">
    <Transition name="overlay-shell">
      <div v-if="open" class="overlay-backdrop" @click.self="emit('close')">
        <div class="overlay-shell" :class="[widthClass, heightClass]">
          <div class="overlay-head">
            <div class="overlay-title-wrap">
              <slot name="title">
                <div v-if="title" class="overlay-title">{{ title }}</div>
                <div v-if="subtitle" class="overlay-subtitle">{{ subtitle }}</div>
              </slot>
            </div>
            <div class="overlay-actions">
              <slot name="actions" />
              <button class="overlay-close" @click="emit('close')" aria-label="Close">Esc</button>
            </div>
          </div>
          <div class="overlay-body">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.overlay-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-buffer-modal);
  background: rgba(0, 0, 0, 0.42);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.overlay-shell {
  width: min(1100px, 100%);
  height: min(86vh, 860px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 14px;
  border: 1px solid var(--app-overlay-border);
  background: var(--app-overlay-bg);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
}

.overlay-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.overlay-title-wrap {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.overlay-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--app-text-primary);
}

.overlay-subtitle {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.overlay-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.overlay-close {
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  color: #94a0c0;
  cursor: pointer;
  font-size: 11px;
  padding: 6px 10px;
}

.overlay-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.overlay-shell-enter-active,
.overlay-shell-leave-active {
  transition: opacity 0.16s ease;
}

.overlay-shell-enter-from,
.overlay-shell-leave-to {
  opacity: 0;
}
</style>
