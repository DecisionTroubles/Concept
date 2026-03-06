<script setup lang="ts">
import { computed } from 'vue'
import { Puzzle, Sparkles } from 'lucide-vue-next'
import type { Node } from '@/bindings'
import type { NodeWorkspaceSlot } from '@/core/nodeExtensions'
import { appKernel } from '@/core/kernel'

const props = defineProps<{
  node: Node
  slot: NodeWorkspaceSlot
  extensionId?: string
  extensionIds?: string[]
}>()

const extensions = computed(() => {
  const allowedIds = props.extensionIds?.length
    ? new Set(props.extensionIds)
    : props.extensionId
      ? new Set([props.extensionId])
      : null

  return appKernel.listNodeWorkspaceExtensions().filter(extension => {
    if (extension.slot !== props.slot) return false
    if (!allowedIds) return true
    return allowedIds.has(extension.id)
  })
})
</script>

<template>
  <div class="extension-outlet">
    <article v-for="extension in extensions" :key="extension.id" class="extension-card">
      <component
        :is="extension.component"
        v-if="extension.component"
        :node="node"
      />
      <template v-else>
        <div class="extension-head">
          <div class="extension-title-row">
            <Puzzle :size="14" />
            <strong>{{ extension.title }}</strong>
          </div>
          <span class="extension-badge">
            <Sparkles :size="12" />
            Plugin slot
          </span>
        </div>
        <p>{{ extension.description }}</p>
        <div class="extension-meta">Slot: {{ extension.slot }} · Node: {{ node.title }}</div>
      </template>
    </article>
  </div>
</template>

<style scoped>
.extension-outlet {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.extension-card {
  border-radius: 14px;
  border: 1px dashed color-mix(in srgb, var(--app-accent) 36%, rgba(255, 255, 255, 0.1));
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--app-accent) 7%, transparent), rgba(255, 255, 255, 0.02));
  padding: 14px;
}

.extension-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.extension-title-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--app-text-primary);
}

.extension-title-row strong {
  font-size: 13px;
}

.extension-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 10px;
  color: var(--app-accent);
  border-radius: 999px;
  padding: 4px 8px;
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 26%, transparent);
}

.extension-card p {
  margin: 10px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--app-text-secondary);
}

.extension-meta {
  margin-top: 10px;
  font-size: 11px;
  color: #aab3ca;
}
</style>
