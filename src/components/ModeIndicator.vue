<script setup lang="ts">
import { Navigation, Wind, GitBranch } from 'lucide-vue-next'

const { mode } = useEditorMode()
</script>

<template>
  <div class="mode-indicator">
    <div :class="['mode-pill', `mode-${mode}`]">
      <Navigation v-if="mode === 'normal'" :size="12" />
      <Wind v-else-if="mode === 'fly'" :size="12" />
      <GitBranch v-else-if="mode === 'graph'" :size="12" />
      <span class="mode-label">{{ mode.toUpperCase() }}</span>
    </div>
    <div class="mode-hint">
      <span v-if="mode === 'normal'">F=fly · G=graph · C/Space focus · Q/E overlays · B=pinned · M=map</span>
      <span v-else-if="mode === 'fly'">WASD move · R/V vertical · Esc exit</span>
      <span v-else-if="mode === 'graph'">Tab/1-9 · H/L orbit · J/K tilt · I/U zoom · C/Space focus · Q/E overlays</span>
    </div>
  </div>
</template>

<style scoped>
.mode-indicator {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 200;
  pointer-events: none;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.mode-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 14px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  font-family: system-ui, sans-serif;
}

.mode-normal {
  background: rgba(120, 130, 170, 0.18);
  color: #8090b0;
  border: 1px solid rgba(120, 130, 170, 0.3);
}

.mode-fly {
  background: rgba(96, 165, 250, 0.18);
  color: #60a5fa;
  border: 1px solid rgba(96, 165, 250, 0.4);
}

.mode-graph {
  background: rgba(52, 211, 153, 0.18);
  color: #34d399;
  border: 1px solid rgba(52, 211, 153, 0.4);
}

.mode-hint {
  font-size: 10px;
  color: rgba(120, 130, 170, 0.6);
  font-family: system-ui, sans-serif;
  white-space: nowrap;
}
</style>
