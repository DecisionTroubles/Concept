<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Node } from '@/bindings'
import { useTauRPC } from '@/composables/useTauRPC'

const props = defineProps<{
  node: Node
}>()

const EXTENSION_KEY = 'node-notes'

const notes = ref('')
const isLoading = ref(false)
const isSaving = ref(false)
const lastSavedAt = ref<string | null>(null)

const saveLabel = computed(() => {
  if (isSaving.value) return 'Saving...'
  if (lastSavedAt.value) return `Saved ${new Date(Number(lastSavedAt.value) * 1000).toLocaleString()}`
  return 'Not saved yet'
})

async function loadNotes() {
  isLoading.value = true
  try {
    const rows = await useTauRPC().get_node_extension_data(props.node.id, EXTENSION_KEY)
    if (rows[0]) {
      const parsed = JSON.parse(rows[0].data_json) as { notes?: string }
      notes.value = typeof parsed.notes === 'string' ? parsed.notes : ''
      lastSavedAt.value = rows[0].updated_at
    } else {
      notes.value = ''
      lastSavedAt.value = null
    }
  } catch {
    notes.value = ''
    lastSavedAt.value = null
  } finally {
    isLoading.value = false
  }
}

async function saveNotes() {
  isSaving.value = true
  try {
    const row = await useTauRPC().set_node_extension_data(
      props.node.id,
      EXTENSION_KEY,
      JSON.stringify({ notes: notes.value })
    )
    lastSavedAt.value = row.updated_at
  } finally {
    isSaving.value = false
  }
}

watch(
  () => props.node.id,
  () => {
    void loadNotes()
  },
  { immediate: true }
)
</script>

<template>
  <div class="notes-extension">
    <div class="notes-head">
      <div>
        <div class="notes-title">Node Notes</div>
        <div class="notes-subtitle">Private extension-backed notes for this node.</div>
      </div>
      <div class="notes-status">{{ isLoading ? 'Loading...' : saveLabel }}</div>
    </div>

    <textarea
      v-model="notes"
      class="notes-input"
      placeholder="Write node-specific notes, observations, reminders, or future plugin ideas here."
    />

    <div class="notes-actions">
      <button class="notes-save" @click="saveNotes">Save notes</button>
    </div>
  </div>
</template>

<style scoped>
.notes-extension {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.notes-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.notes-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--app-text-primary);
}

.notes-subtitle,
.notes-status {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.notes-input {
  width: 100%;
  min-height: 180px;
  resize: vertical;
  padding: 12px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.03);
  color: var(--app-text-primary);
}

.notes-actions {
  display: flex;
  justify-content: flex-end;
}

.notes-save {
  border: 1px solid color-mix(in srgb, var(--app-accent) 32%, transparent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  border-radius: 999px;
  padding: 8px 12px;
  font-size: 12px;
  cursor: pointer;
}
</style>
