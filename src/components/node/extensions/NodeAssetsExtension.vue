<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Node } from '@/bindings'
import { useTauRPC } from '@/composables/useTauRPC'

type AssetEntry = {
  id: string
  kind: 'image' | 'audio' | 'music' | 'link' | 'file'
  title: string
  source: string
}

const props = defineProps<{
  node: Node
}>()

const EXTENSION_KEY = 'node-assets'

const assets = ref<AssetEntry[]>([])
const isLoading = ref(false)
const isSaving = ref(false)
const lastSavedAt = ref<string | null>(null)

const draftKind = ref<AssetEntry['kind']>('image')
const draftTitle = ref('')
const draftSource = ref('')

const saveLabel = computed(() => {
  if (isSaving.value) return 'Saving...'
  if (lastSavedAt.value) return `Saved ${new Date(Number(lastSavedAt.value) * 1000).toLocaleString()}`
  return 'Not saved yet'
})

async function loadAssets() {
  isLoading.value = true
  try {
    const rows = await useTauRPC().get_node_extension_data(props.node.id, EXTENSION_KEY)
    if (rows[0]) {
      const parsed = JSON.parse(rows[0].data_json) as { assets?: AssetEntry[] }
      assets.value = Array.isArray(parsed.assets) ? parsed.assets : []
      lastSavedAt.value = rows[0].updated_at
    } else {
      assets.value = []
      lastSavedAt.value = null
    }
  } catch {
    assets.value = []
    lastSavedAt.value = null
  } finally {
    isLoading.value = false
  }
}

async function persistAssets() {
  isSaving.value = true
  try {
    const row = await useTauRPC().set_node_extension_data(
      props.node.id,
      EXTENSION_KEY,
      JSON.stringify({ assets: assets.value })
    )
    lastSavedAt.value = row.updated_at
  } finally {
    isSaving.value = false
  }
}

async function addAsset() {
  const title = draftTitle.value.trim()
  const source = draftSource.value.trim()
  if (!title || !source) return
  assets.value = [
    ...assets.value,
    {
      id: crypto.randomUUID(),
      kind: draftKind.value,
      title,
      source,
    },
  ]
  draftTitle.value = ''
  draftSource.value = ''
  await persistAssets()
}

async function removeAsset(id: string) {
  assets.value = assets.value.filter(asset => asset.id !== id)
  await persistAssets()
}

watch(
  () => props.node.id,
  () => {
    void loadAssets()
  },
  { immediate: true }
)
</script>

<template>
  <div class="assets-extension">
    <div class="assets-head">
      <div>
        <div class="assets-title">Node Assets</div>
        <div class="assets-subtitle">Images, audio, music, files, and useful links attached to this node.</div>
      </div>
      <div class="assets-status">{{ isLoading ? 'Loading...' : saveLabel }}</div>
    </div>

    <div class="asset-form">
      <select v-model="draftKind">
        <option value="image">Image</option>
        <option value="audio">Audio</option>
        <option value="music">Music</option>
        <option value="link">Link</option>
        <option value="file">File</option>
      </select>
      <input v-model="draftTitle" type="text" placeholder="Asset title" />
      <input v-model="draftSource" type="text" placeholder="URL or path" />
      <button class="asset-add" @click="addAsset">Add asset</button>
    </div>

    <div v-if="assets.length === 0" class="assets-empty">
      No assets attached yet.
    </div>

    <div v-else class="asset-list">
      <article v-for="asset in assets" :key="asset.id" class="asset-card">
        <div class="asset-main">
          <div class="asset-title-row">
            <strong>{{ asset.title }}</strong>
            <span class="asset-kind">{{ asset.kind }}</span>
          </div>
          <div class="asset-source">{{ asset.source }}</div>
        </div>
        <button class="asset-remove" @click="removeAsset(asset.id)">Remove</button>
      </article>
    </div>
  </div>
</template>

<style scoped>
.assets-extension {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.assets-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.assets-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--app-text-primary);
}

.assets-subtitle,
.assets-status,
.assets-empty,
.asset-source {
  font-size: 11px;
  color: var(--app-text-secondary);
}

.asset-form {
  display: grid;
  grid-template-columns: 140px 1fr 1.2fr auto;
  gap: 10px;
}

.asset-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.asset-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
}

.asset-main {
  min-width: 0;
}

.asset-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.asset-title-row strong {
  color: var(--app-text-primary);
  font-size: 13px;
}

.asset-kind {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--app-accent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 26%, transparent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border-radius: 999px;
  padding: 3px 8px;
}

.asset-add,
.asset-remove {
  border: 1px solid color-mix(in srgb, var(--app-accent) 32%, transparent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  border-radius: 999px;
  padding: 8px 12px;
  font-size: 12px;
  cursor: pointer;
}

@media (max-width: 860px) {
  .asset-form {
    grid-template-columns: 1fr;
  }
}
</style>
