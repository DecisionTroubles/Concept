<script setup lang="ts">
import { computed } from 'vue'
import { useEventListener } from '@vueuse/core'
import OverlayShell from '@/components/ui/OverlayShell.vue'

const graphStore = useGraphStore()
const settings = useSettings()

const validWorlds = computed(() => graphStore.worldPacks.filter(world => world.valid))
const registryByWorldId = computed(() => {
  const map = new Map<string, string>()
  for (const entry of graphStore.packRegistry) {
    const worldId = entry.pack_info?.world_id
    if (worldId) map.set(worldId, entry.install_status)
  }
  return map
})

function openPicker() {
  graphStore.openWorldPicker()
}

function closePicker() {
  graphStore.closeWorldPicker()
}

async function openWorld(worldId: string) {
  await graphStore.selectWorld(worldId)
}

async function reloadCurrentWorld() {
  await graphStore.reloadActiveWorld()
}

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    if (isInput) return

    const key = e.key.toLowerCase()
    if (key === settings.keys.worldPicker) {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.toggleWorldPicker()
      return
    }

    if (graphStore.worldPickerOpen && e.key === 'Escape') {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.closeWorldPicker()
    }
  },
  { capture: true }
)
</script>

<template>
  <button
    class="world-picker-btn"
    :class="{ active: graphStore.worldPickerOpen }"
    :title="`World picker (${settings.keys.worldPicker.toUpperCase()})`"
    @click="openPicker"
  >
    <span class="world-picker-btn-label">Worlds</span>
    <span class="world-picker-btn-key">{{ settings.keys.worldPicker.toUpperCase() }}</span>
  </button>

  <OverlayShell
    :open="graphStore.worldPickerOpen"
    title="World Picker"
    subtitle="Choose which world pack to load into the app"
    width-class="world-picker-shell"
    height-class="world-picker-shell"
    @close="closePicker"
  >
    <div class="world-picker-layout">
      <section class="world-picker-summary">
        <div class="summary-block">
          <div class="summary-label">Active world</div>
          <strong>{{ graphStore.worldConfig?.name ?? 'No world loaded' }}</strong>
          <span>{{ graphStore.worldConfig?.id ?? 'No active world in database' }}</span>
        </div>

        <div class="summary-block">
          <div class="summary-label">Detected packs</div>
          <strong>{{ graphStore.worldPacks.length }}</strong>
          <span>{{ validWorlds.length }} valid</span>
        </div>

        <div class="summary-copy">
          Installed worlds are loaded from your local pack library. Add GitHub pack sources from Settings to bring in more worlds.
        </div>

        <div class="summary-actions">
          <button class="summary-btn" @click="reloadCurrentWorld">Reload active world</button>
          <button class="summary-btn subtle" @click="closePicker">Close</button>
        </div>
      </section>

      <section class="world-picker-list">
        <article
          v-for="world in graphStore.worldPacks"
          :key="world.pack_path"
          class="world-card"
          :class="{ active: world.is_active, invalid: !world.valid }"
        >
          <div class="world-card-head">
            <div class="world-card-title">
              <strong>{{ world.world_name ?? 'Invalid pack' }}</strong>
              <span>{{ world.world_id ?? 'Missing world.id' }}</span>
            </div>
            <div class="world-card-badges">
              <span class="world-badge source">{{ world.source_kind }}</span>
              <span v-if="world.world_id && registryByWorldId.get(world.world_id)" class="world-badge source">
                {{ registryByWorldId.get(world.world_id) }}
              </span>
              <span v-if="world.is_loaded" class="world-badge state loaded">Loaded</span>
              <span v-else-if="world.is_active" class="world-badge state selected">Selected</span>
            </div>
          </div>

          <p class="world-card-path">{{ world.pack_path }}</p>
          <p v-if="world.error" class="world-card-error">{{ world.error }}</p>

          <div class="world-card-actions">
            <button
              class="summary-btn"
              :disabled="!world.valid || !world.world_id || world.is_active || graphStore.isLoading"
              @click="world.world_id && openWorld(world.world_id)"
            >
              {{ world.is_active ? 'Current world' : 'Open world' }}
            </button>
          </div>
        </article>
      </section>
    </div>
  </OverlayShell>
</template>

<style scoped>
.world-picker-btn {
  position: fixed;
  top: 14px;
  right: 58px;
  z-index: var(--z-settings-gear);
  height: 32px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--app-overlay-bg) 88%, transparent);
  border: 1px solid var(--app-overlay-border);
  color: var(--app-text-secondary);
  cursor: pointer;
  transition:
    background 0.15s,
    color 0.15s,
    border-color 0.15s;
  backdrop-filter: blur(8px);
}

.world-picker-btn:hover,
.world-picker-btn.active {
  background: color-mix(in srgb, var(--app-accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 35%, transparent);
  color: var(--app-accent);
}

.world-picker-btn-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.world-picker-btn-key {
  font-size: 10px;
  font-family: ui-monospace, 'Cascadia Code', monospace;
  padding: 2px 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.world-picker-layout {
  display: grid;
  grid-template-columns: minmax(240px, 0.7fr) minmax(0, 1.3fr);
  gap: 16px;
  padding: 16px;
}

.world-picker-summary,
.world-card {
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.035);
}

.world-picker-summary {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.summary-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.summary-label {
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.summary-block strong {
  font-size: 18px;
  color: var(--app-text-primary);
}

.summary-block span,
.summary-copy {
  font-size: 12px;
  color: var(--app-text-secondary);
  line-height: 1.5;
}

.summary-actions,
.world-card-actions,
.world-card-badges {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.summary-btn {
  font-size: 12px;
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 13%, transparent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 25%, transparent);
  cursor: pointer;
  padding: 7px 11px;
  border-radius: 8px;
  transition:
    background 0.12s,
    border-color 0.12s,
    opacity 0.12s;
}

.summary-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--app-accent) 20%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 45%, transparent);
}

.summary-btn.subtle {
  color: var(--app-text-secondary);
  background: rgba(255, 255, 255, 0.04);
  border-color: rgba(255, 255, 255, 0.08);
}

.summary-btn:disabled {
  cursor: default;
  opacity: 0.45;
}

.world-picker-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.world-card {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.world-card.active {
  border-color: color-mix(in srgb, var(--app-accent) 42%, transparent);
  background: color-mix(in srgb, var(--app-accent) 8%, transparent);
}

.world-card.invalid {
  border-color: rgba(243, 154, 143, 0.34);
}

.world-card-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.world-card-title {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.world-card-title strong {
  font-size: 14px;
  color: var(--app-text-primary);
}

.world-card-title span,
.world-card-path {
  font-size: 12px;
  color: var(--app-text-secondary);
  word-break: break-all;
}

.world-badge {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid transparent;
}

.world-badge.source {
  color: var(--app-text-secondary);
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.08);
}

.world-badge.state.loaded,
.world-badge.state.selected {
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 28%, transparent);
}

.world-card-error {
  margin: 0;
  font-size: 12px;
  color: #f39a8f;
}

@media (max-width: 900px) {
  .world-picker-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .world-picker-btn {
    right: 54px;
    padding: 0 8px;
  }

  .world-picker-btn-label {
    display: none;
  }
}
</style>
