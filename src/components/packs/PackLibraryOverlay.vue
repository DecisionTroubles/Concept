<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useEventListener } from '@vueuse/core'
import { open } from '@tauri-apps/plugin-dialog'
import { ArrowUpRight, Download, FolderOpen, Github, GitBranch, Plus, RefreshCcw, Save, Search, Trash2 } from 'lucide-vue-next'
import type { GitHubPackSourceInput, LocalPackSourceInput, PackRegistryEntry, WorldPackInfo } from '@/bindings'
import OverlayShell from '@/components/ui/OverlayShell.vue'

type LibrarySection = 'remote' | 'local'
type AddMode = 'github' | 'local'
type Selection =
  | { kind: 'source'; id: string }
  | { kind: 'world'; packPath: string }
  | null

const graphStore = useGraphStore()
const settings = useSettings()

const activeSection = ref<LibrarySection>('remote')
const selection = ref<Selection>(null)
const showAddModal = ref(false)
const addMode = ref<AddMode>('github')
const remoteDraft = ref<GitHubPackSourceInput>({
  id: '',
  name: '',
  repo: '',
  path: '',
  branch: 'main',
  pinned_ref: null,
  enabled: true,
})
const localDraft = ref<LocalPackSourceInput>({
  id: '',
  name: '',
  path: '',
  enabled: true,
})

const remoteSources = computed(() =>
  [...graphStore.packRegistry]
    .filter(entry => entry.source.provider === 'github')
    .sort((a, b) => a.source.name.localeCompare(b.source.name))
)

const trackedLocalSources = computed(() =>
  [...graphStore.packRegistry]
    .filter(entry => entry.source.provider === 'local')
    .sort((a, b) => a.source.name.localeCompare(b.source.name))
)

const localWorlds = computed(() =>
  [...graphStore.worldPacks].sort((a, b) => (a.world_name ?? a.pack_path).localeCompare(b.world_name ?? b.pack_path))
)

const visibleLocalWorlds = computed(() => {
  const trackedPaths = new Set(
    trackedLocalSources.value
      .map(entry => entry.pack_info?.pack_path)
      .filter((path): path is string => Boolean(path))
  )
  return localWorlds.value.filter(world => !trackedPaths.has(world.pack_path))
})

const selectedSourceId = computed(() => (selection.value?.kind === 'source' ? selection.value.id : null))
const selectedWorldPath = computed(() => (selection.value?.kind === 'world' ? selection.value.packPath : null))

const selectedSource = computed<PackRegistryEntry | null>(() => {
  if (!selectedSourceId.value) return null
  return graphStore.packRegistry.find(entry => entry.source.id === selectedSourceId.value) ?? null
})

const selectedWorld = computed<WorldPackInfo | null>(() => {
  if (!selectedWorldPath.value) return null
  return visibleLocalWorlds.value.find(world => world.pack_path === selectedWorldPath.value) ?? null
})

const selectedRemoteSource = computed(() => (
  selectedSource.value?.source.provider === 'github' ? selectedSource.value : null
))

const selectedLocalSource = computed(() => (
  selectedSource.value?.source.provider === 'local' ? selectedSource.value : null
))

function resetRemoteDraft() {
  remoteDraft.value = {
    id: '',
    name: '',
    repo: '',
    path: '',
    branch: 'main',
    pinned_ref: null,
    enabled: true,
  }
}

function resetLocalDraft() {
  localDraft.value = {
    id: '',
    name: '',
    path: '',
    enabled: true,
  }
}

function sourceToDraft(entry: PackRegistryEntry): GitHubPackSourceInput {
  return {
    id: entry.source.id,
    name: entry.source.name,
    repo: entry.source.repo,
    path: entry.source.path,
    branch: entry.source.branch,
    pinned_ref: entry.source.pinned_ref,
    enabled: entry.source.enabled,
  }
}

function localSourceToDraft(entry: PackRegistryEntry): LocalPackSourceInput {
  return {
    id: entry.source.id,
    name: entry.source.name,
    path: entry.source.path,
    enabled: entry.source.enabled,
  }
}

function openAddModal(mode: AddMode) {
  addMode.value = mode
  if (mode === 'github') resetRemoteDraft()
  else resetLocalDraft()
  showAddModal.value = true
}

function closeAddModal() {
  showAddModal.value = false
}

async function browseLocalPackPath() {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: localDraft.value.path.trim() || undefined,
    title: 'Select local pack folder',
  })
  if (typeof selected === 'string' && selected.trim()) {
    localDraft.value.path = selected
    await hydrateLocalDraftFromPath(selected)
  }
}

async function hydrateLocalDraftFromPath(path: string) {
  const trimmed = path.trim()
  if (!trimmed) return
  try {
    const probe = await graphStore.inspectLocalPackPath(trimmed)
    localDraft.value.path = trimmed
    if (!localDraft.value.id.trim()) localDraft.value.id = probe.suggested_id
    if (!localDraft.value.name.trim()) localDraft.value.name = probe.suggested_name
  } catch {
    localDraft.value.path = trimmed
  }
}

function selectSource(id: string) {
  const provider = graphStore.packRegistry.find(entry => entry.source.id === id)?.source.provider
  activeSection.value = provider === 'local' ? 'local' : 'remote'
  selection.value = { kind: 'source', id }
}

function selectWorld(packPath: string) {
  activeSection.value = 'local'
  selection.value = { kind: 'world', packPath }
}

function ensureSelection() {
  if (activeSection.value === 'remote') {
    if (selectedRemoteSource.value) return
    const first = remoteSources.value[0]
    selection.value = first ? { kind: 'source', id: first.source.id } : null
    return
  }
  if (selectedLocalSource.value || selectedWorld.value) return
  const firstLocalSource = trackedLocalSources.value[0]
  if (firstLocalSource) {
    selection.value = { kind: 'source', id: firstLocalSource.source.id }
    return
  }
  if (selectedWorld.value) return
  const first = visibleLocalWorlds.value[0]
  selection.value = first ? { kind: 'world', packPath: first.pack_path } : null
}

async function saveNewSource() {
  if (addMode.value === 'local') {
    const payload: LocalPackSourceInput = {
      id: localDraft.value.id.trim(),
      name: localDraft.value.name.trim(),
      path: localDraft.value.path.trim(),
      enabled: localDraft.value.enabled,
    }
    await graphStore.addLocalPackSource(payload)
    const entry = await graphStore.refreshPackSource(payload.id)
    closeAddModal()
    activeSection.value = 'local'
    selection.value = { kind: 'source', id: payload.id }
    await openWorld(entry.pack_info?.world_id)
    return
  }
  const payload: GitHubPackSourceInput = {
    id: remoteDraft.value.id.trim(),
    name: remoteDraft.value.name.trim(),
    repo: remoteDraft.value.repo.trim(),
    path: remoteDraft.value.path.trim(),
    branch: remoteDraft.value.branch.trim() || 'main',
    pinned_ref: remoteDraft.value.pinned_ref?.trim() ? remoteDraft.value.pinned_ref.trim() : null,
    enabled: remoteDraft.value.enabled,
  }
  await graphStore.addGitHubPackSource(payload)
  const entry = await graphStore.installPackSource(payload.id)
  closeAddModal()
  selectSource(payload.id)
  await openWorld(entry.pack_info?.world_id)
}

async function saveSelectedSource() {
  if (!selectedSource.value) return
  const sourceId = selectedSource.value.source.id
  if (selectedSource.value.source.provider === 'local') {
    const payload: LocalPackSourceInput = {
      id: localDraft.value.id.trim(),
      name: localDraft.value.name.trim(),
      path: localDraft.value.path.trim(),
      enabled: localDraft.value.enabled,
    }
    await graphStore.updateLocalPackSource(sourceId, payload)
    const entry = await graphStore.refreshPackSource(sourceId)
    activeSection.value = 'local'
    selection.value = { kind: 'source', id: sourceId }
    await openWorld(entry.pack_info?.world_id)
    return
  }
  const payload: GitHubPackSourceInput = {
    id: remoteDraft.value.id.trim(),
    name: remoteDraft.value.name.trim(),
    repo: remoteDraft.value.repo.trim(),
    path: remoteDraft.value.path.trim(),
    branch: remoteDraft.value.branch.trim() || 'main',
    pinned_ref: remoteDraft.value.pinned_ref?.trim() ? remoteDraft.value.pinned_ref.trim() : null,
    enabled: remoteDraft.value.enabled,
  }
  await graphStore.updatePackSource(sourceId, payload)
  selectSource(sourceId)
}

async function installAndOpenSource(sourceId: string) {
  const entry = await graphStore.installPackSource(sourceId)
  await openWorld(entry.pack_info?.world_id)
}

async function refreshAndOpenSource(sourceId: string) {
  const entry = await graphStore.refreshPackSource(sourceId)
  await openWorld(entry.pack_info?.world_id)
}

async function removeSelectedSource() {
  if (!selectedSource.value) return
  await graphStore.removePackSource(selectedSource.value.source.id)
  ensureSelection()
}

async function deleteSelectedLocalWorld() {
  if (!selectedWorld.value) return
  await graphStore.deleteLocalWorld(selectedWorld.value.pack_path)
  ensureSelection()
}

async function openWorld(worldId: string | null | undefined) {
  if (!worldId) return
  await graphStore.selectWorld(worldId)
}

function sourceStatusText(entry: PackRegistryEntry): string {
  if (entry.source.provider === 'local') return entry.source.path
  const parts = [entry.source.repo]
  if (entry.source.branch) parts.push(entry.source.branch)
  return parts.join(' · ')
}

function worldStatusText(world: WorldPackInfo): string {
  const flags: string[] = [world.source_kind]
  if (world.is_active) flags.push('active')
  else if (world.is_loaded) flags.push('loaded')
  return flags.join(' · ')
}

watch(
  selectedSource,
  entry => {
    if (!entry) return
    if (entry.source.provider === 'local') {
      localDraft.value = localSourceToDraft(entry)
      return
    }
    remoteDraft.value = sourceToDraft(entry)
  },
  { immediate: true }
)

watch(activeSection, () => {
  ensureSelection()
})

watch([remoteSources, trackedLocalSources, visibleLocalWorlds], () => {
  ensureSelection()
}, { immediate: true })

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    if (isInput) return
    if (e.key.toLowerCase() === settings.keys.packsBuffer) {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.togglePackLibrary()
      return
    }
    if (graphStore.packLibraryOpen && e.key === 'Escape') {
      e.preventDefault()
      e.stopImmediatePropagation()
      if (showAddModal.value) {
        closeAddModal()
        return
      }
      graphStore.closePackLibrary()
    }
  },
  { capture: true }
)
</script>

<template>
  <button
    class="packs-btn"
    :class="{ active: graphStore.packLibraryOpen }"
    :title="`Pack library (${settings.keys.packsBuffer.toUpperCase()})`"
    @click="graphStore.togglePackLibrary()"
  >
    <span class="packs-btn-label">Packs</span>
    <span class="packs-btn-key">{{ settings.keys.packsBuffer.toUpperCase() }}</span>
  </button>

  <OverlayShell
    :open="graphStore.packLibraryOpen"
    title="Pack Library"
    subtitle="Track GitHub sources and your own local pack folders in one place"
    width-class="pack-library-shell"
    height-class="pack-library-shell"
    @close="graphStore.closePackLibrary()"
  >
    <div class="pack-library-layout">
      <aside class="sidebar">
        <div class="sidebar-topbar">
          <div class="section-switch">
            <button class="switch-btn" :class="{ active: activeSection === 'remote' }" @click="activeSection = 'remote'">
              Remote
              <span>{{ remoteSources.length }}</span>
            </button>
            <button class="switch-btn" :class="{ active: activeSection === 'local' }" @click="activeSection = 'local'">
              Local
              <span>{{ localWorlds.length }}</span>
            </button>
          </div>
          <div class="sidebar-actions">
            <button class="icon-btn" title="Refresh pack library" @click="graphStore.refreshPackRegistry()">
              <RefreshCcw :size="14" />
            </button>
            <button
              class="icon-btn accent"
              :title="activeSection === 'remote' ? 'Add GitHub source' : 'Add local source'"
              @click="openAddModal(activeSection === 'remote' ? 'github' : 'local')"
            >
              <Plus :size="14" />
            </button>
          </div>
        </div>

        <div v-if="activeSection === 'remote'" class="list-wrap">
          <button
            v-for="entry in remoteSources"
            :key="entry.source.id"
            class="list-item"
            :class="{ active: selectedSourceId === entry.source.id }"
            @click="selectSource(entry.source.id)"
          >
            <div class="list-item-head">
              <strong>{{ entry.source.name }}</strong>
              <span class="item-badge">{{ entry.install_status }}</span>
            </div>
            <div class="list-item-sub">{{ sourceStatusText(entry) }}</div>
            <div v-if="entry.pack_info?.world_name" class="list-item-meta">{{ entry.pack_info.world_name }}</div>
          </button>
          <div v-if="remoteSources.length === 0" class="empty-state">
            No tracked sources yet.
          </div>
        </div>

        <div v-else class="list-wrap">
          <div v-if="trackedLocalSources.length" class="sidebar-section-label">Tracked local sources</div>
          <button
            v-for="entry in trackedLocalSources"
            :key="entry.source.id"
            class="list-item"
            :class="{ active: selectedSourceId === entry.source.id }"
            @click="selectSource(entry.source.id)"
          >
            <div class="list-item-head">
              <strong>{{ entry.source.name }}</strong>
              <span class="item-badge">{{ entry.install_status }}</span>
            </div>
            <div class="list-item-sub">{{ sourceStatusText(entry) }}</div>
            <div v-if="entry.pack_info?.world_name" class="list-item-meta">{{ entry.pack_info.world_name }}</div>
          </button>
          <div v-if="visibleLocalWorlds.length" class="sidebar-section-label">Detected local packs</div>
          <button
            v-for="world in visibleLocalWorlds"
            :key="world.pack_path"
            class="list-item"
            :class="{ active: selectedWorldPath === world.pack_path }"
            @click="selectWorld(world.pack_path)"
          >
            <div class="list-item-head">
              <strong>{{ world.world_name ?? 'Invalid pack' }}</strong>
              <span class="item-badge">{{ world.source_kind }}</span>
            </div>
            <div class="list-item-sub">{{ world.world_id ?? world.pack_path }}</div>
            <div class="list-item-meta">{{ worldStatusText(world) }}</div>
          </button>
          <div v-if="trackedLocalSources.length === 0 && visibleLocalWorlds.length === 0" class="empty-state">
            No tracked local sources or discovered local packs yet.
          </div>
        </div>
      </aside>

      <section class="detail-pane">
        <template v-if="activeSection === 'remote' && selectedRemoteSource">
          <div class="detail-hero">
            <div>
              <div class="detail-kicker">Tracked Source</div>
              <h2>{{ selectedRemoteSource.source.name }}</h2>
              <p>{{ selectedRemoteSource.source.repo }}/{{ selectedRemoteSource.source.path }}</p>
            </div>
            <div class="detail-hero-actions">
              <button class="action-btn primary" @click="installAndOpenSource(selectedRemoteSource.source.id)">
                <Download :size="14" />
                Pull latest
              </button>
              <button class="action-btn" @click="graphStore.checkPackSourceUpdates(selectedRemoteSource.source.id)">
                <ArrowUpRight :size="14" />
                Check updates
              </button>
              <button class="action-btn" @click="refreshAndOpenSource(selectedRemoteSource.source.id)">
                <RefreshCcw :size="14" />
                Refresh
              </button>
            </div>
          </div>

          <div class="detail-grid">
            <section class="detail-card">
              <div class="card-head">
                <strong>Source</strong>
                <span>Edit branch, path, and pin without leaving the source view.</span>
              </div>
              <div class="form-grid">
                <label class="field">
                  <span>Pack id</span>
                  <input v-model="remoteDraft.id" type="text" />
                </label>
                <label class="field">
                  <span>Name</span>
                  <input v-model="remoteDraft.name" type="text" />
                </label>
                <label class="field">
                  <span>GitHub repo</span>
                  <input v-model="remoteDraft.repo" type="text" />
                </label>
                <label class="field">
                  <span>Folder path</span>
                  <input v-model="remoteDraft.path" type="text" />
                </label>
                <label class="field">
                  <span>Branch</span>
                  <div class="field-with-icon">
                    <GitBranch :size="14" />
                    <input v-model="remoteDraft.branch" type="text" />
                  </div>
                </label>
                <label class="field">
                  <span>Pinned ref</span>
                  <input v-model="remoteDraft.pinned_ref" type="text" placeholder="optional tag or sha" />
                </label>
              </div>
              <label class="toggle-row">
                <span>Enabled</span>
                <input v-model="remoteDraft.enabled" type="checkbox" />
              </label>
              <div class="detail-actions">
                <button class="action-btn primary" @click="saveSelectedSource()">
                  <Save :size="14" />
                  Save changes
                </button>
                <button class="action-btn danger" @click="removeSelectedSource()">
                  <Trash2 :size="14" />
                  Remove
                </button>
              </div>
            </section>

            <section class="detail-card">
              <div class="card-head">
                <strong>Status</strong>
                <span>Small set of state you actually need while managing the source.</span>
              </div>
              <div class="meta-stack">
                <div class="meta-row">
                  <span>Install status</span>
                  <strong>{{ selectedRemoteSource.install_status }}</strong>
                </div>
                <div class="meta-row">
                  <span>Installed version</span>
                  <strong>{{ selectedRemoteSource.source.installed_version ?? 'none' }}</strong>
                </div>
                <div class="meta-row">
                  <span>Last checked</span>
                  <strong>{{ selectedRemoteSource.source.last_checked_at ?? 'never' }}</strong>
                </div>
                <div class="meta-row">
                  <span>Last installed</span>
                  <strong>{{ selectedRemoteSource.source.last_installed_at ?? 'never' }}</strong>
                </div>
                <div v-if="selectedRemoteSource.pack_info?.world_name" class="meta-row">
                  <span>Installed world</span>
                  <strong>{{ selectedRemoteSource.pack_info.world_name }}</strong>
                </div>
              </div>
              <div class="detail-actions">
                <button
                  v-if="selectedRemoteSource.pack_info?.world_id"
                  class="action-btn"
                  @click="openWorld(selectedRemoteSource.pack_info.world_id)"
                >
                  <FolderOpen :size="14" />
                  Open world
                </button>
              </div>
              <div v-if="selectedRemoteSource.last_error" class="error-box">
                {{ selectedRemoteSource.last_error }}
              </div>
            </section>
          </div>
        </template>

        <template v-else-if="activeSection === 'local' && selectedLocalSource">
          <div class="detail-hero">
            <div>
              <div class="detail-kicker">Tracked Local Pack</div>
              <h2>{{ selectedLocalSource.source.name }}</h2>
              <p>{{ selectedLocalSource.source.path }}</p>
            </div>
            <div class="detail-hero-actions">
              <button class="action-btn primary" @click="refreshAndOpenSource(selectedLocalSource.source.id)">
                <RefreshCcw :size="14" />
                Sync local pack
              </button>
              <button
                v-if="selectedLocalSource.pack_info?.world_id"
                class="action-btn"
                @click="openWorld(selectedLocalSource.pack_info.world_id)"
              >
                <FolderOpen :size="14" />
                Open world
              </button>
            </div>
          </div>

          <div class="detail-grid">
            <section class="detail-card">
              <div class="card-head">
                <strong>Local Source</strong>
                <span>Track any folder or direct `pack.json` path, then sync it into the app-managed local library.</span>
              </div>
              <div class="form-grid detail-grid-single">
                <label class="field">
                  <span>Pack id</span>
                  <input v-model="localDraft.id" type="text" />
                </label>
                <label class="field">
                  <span>Name</span>
                  <input v-model="localDraft.name" type="text" />
                </label>
                <label class="field field-span-2">
                  <span>Local pack path</span>
                  <div class="field-with-action">
                    <FolderOpen :size="14" />
                    <input
                      v-model="localDraft.path"
                      type="text"
                      placeholder="C:\\packs\\japanese-pack or C:\\packs\\japanese-pack\\pack.json"
                      @blur="hydrateLocalDraftFromPath(localDraft.path)"
                    />
                    <button class="inline-action-btn" type="button" @click="browseLocalPackPath()">
                      <Search :size="14" />
                      Browse
                    </button>
                  </div>
                </label>
              </div>
              <label class="toggle-row">
                <span>Enabled</span>
                <input v-model="localDraft.enabled" type="checkbox" />
              </label>
              <div class="detail-actions">
                <button class="action-btn primary" @click="saveSelectedSource()">
                  <Save :size="14" />
                  Save and sync
                </button>
                <button class="action-btn danger" @click="removeSelectedSource()">
                  <Trash2 :size="14" />
                  Remove
                </button>
              </div>
            </section>

            <section class="detail-card">
              <div class="card-head">
                <strong>Sync State</strong>
                <span>Shows whether the managed local copy is available to the Projects picker.</span>
              </div>
              <div class="meta-stack">
                <div class="meta-row">
                  <span>Status</span>
                  <strong>{{ selectedLocalSource.install_status }}</strong>
                </div>
                <div class="meta-row">
                  <span>Last checked</span>
                  <strong>{{ selectedLocalSource.source.last_checked_at ?? 'never' }}</strong>
                </div>
                <div class="meta-row">
                  <span>Last synced</span>
                  <strong>{{ selectedLocalSource.source.last_installed_at ?? 'never' }}</strong>
                </div>
                <div class="meta-row">
                  <span>Managed world</span>
                  <strong>{{ selectedLocalSource.pack_info?.world_name ?? 'not synced yet' }}</strong>
                </div>
              </div>
              <div v-if="selectedLocalSource.last_error" class="error-box">
                {{ selectedLocalSource.last_error }}
              </div>
            </section>
          </div>
        </template>

        <template v-else-if="activeSection === 'local' && selectedWorld">
          <div class="detail-hero">
            <div>
              <div class="detail-kicker">Local Pack</div>
              <h2>{{ selectedWorld.world_name ?? 'Invalid pack' }}</h2>
              <p>{{ selectedWorld.pack_path }}</p>
            </div>
            <div class="detail-hero-actions">
              <button
                class="action-btn primary"
                :disabled="!selectedWorld.valid || !selectedWorld.world_id || selectedWorld.is_active || graphStore.isLoading"
                @click="openWorld(selectedWorld.world_id)"
              >
                <FolderOpen :size="14" />
                {{ selectedWorld.is_active ? 'Current world' : 'Open world' }}
              </button>
              <button class="action-btn" @click="graphStore.reloadActiveWorld()">
                <RefreshCcw :size="14" />
                Reload active
              </button>
              <button
                v-if="selectedWorld.source_kind === 'local'"
                class="action-btn danger"
                :disabled="graphStore.isLoading"
                @click="deleteSelectedLocalWorld()"
              >
                <Trash2 :size="14" />
                Delete local copy
              </button>
            </div>
          </div>

          <div class="detail-grid detail-grid-single">
            <section class="detail-card">
              <div class="card-head">
                <strong>Pack State</strong>
                <span>Local and installed copies stay here so you can jump back from local to remote at any time.</span>
              </div>
              <div class="meta-stack">
                <div class="meta-row">
                  <span>World id</span>
                  <strong>{{ selectedWorld.world_id ?? 'missing' }}</strong>
                </div>
                <div class="meta-row">
                  <span>Source kind</span>
                  <strong>{{ selectedWorld.source_kind }}</strong>
                </div>
                <div class="meta-row">
                  <span>Valid</span>
                  <strong>{{ selectedWorld.valid ? 'yes' : 'no' }}</strong>
                </div>
                <div class="meta-row">
                  <span>Loaded</span>
                  <strong>{{ selectedWorld.is_loaded ? 'yes' : 'no' }}</strong>
                </div>
                <div class="meta-row">
                  <span>Active</span>
                  <strong>{{ selectedWorld.is_active ? 'yes' : 'no' }}</strong>
                </div>
              </div>
              <div v-if="selectedWorld.error" class="error-box">
                {{ selectedWorld.error }}
              </div>
            </section>
          </div>
        </template>

        <div v-else class="detail-empty">
          <strong>No pack selected</strong>
          <span>Choose a remote source or a local pack from the list.</span>
        </div>
      </section>
    </div>
  </OverlayShell>

  <OverlayShell
    :open="showAddModal"
    :title="addMode === 'github' ? 'Add GitHub Source' : 'Add Local Pack Source'"
    :subtitle="addMode === 'github'
      ? 'Create a new tracked GitHub source without crowding the main pack window'
      : 'Track a local pack folder or direct pack.json path without crowding the main pack window'"
    width-class="pack-add-shell"
    @close="closeAddModal()"
  >
    <div class="add-modal-body">
      <div v-if="addMode === 'github'" class="form-grid">
        <label class="field">
          <span>Pack id</span>
          <input v-model="remoteDraft.id" type="text" placeholder="language-core" />
        </label>
        <label class="field">
          <span>Name</span>
          <input v-model="remoteDraft.name" type="text" placeholder="Language Core" />
        </label>
        <label class="field">
          <span>GitHub repo</span>
          <div class="field-with-icon">
            <Github :size="14" />
            <input v-model="remoteDraft.repo" type="text" placeholder="owner/repo" />
          </div>
        </label>
        <label class="field">
          <span>Folder path</span>
          <input v-model="remoteDraft.path" type="text" placeholder="packs/language-core" />
        </label>
        <label class="field">
          <span>Branch</span>
          <div class="field-with-icon">
            <GitBranch :size="14" />
            <input v-model="remoteDraft.branch" type="text" placeholder="main" />
          </div>
        </label>
        <label class="field">
          <span>Pinned ref</span>
          <input v-model="remoteDraft.pinned_ref" type="text" placeholder="optional tag or sha" />
        </label>
      </div>
      <div v-else class="form-grid detail-grid-single">
        <label class="field">
          <span>Pack id</span>
          <input v-model="localDraft.id" type="text" placeholder="japanese-local" />
        </label>
        <label class="field">
          <span>Name</span>
          <input v-model="localDraft.name" type="text" placeholder="Japanese Local" />
        </label>
        <label class="field field-span-2">
          <span>Local pack path</span>
          <div class="field-with-action">
            <FolderOpen :size="14" />
            <input
              v-model="localDraft.path"
              type="text"
              placeholder="C:\\packs\\japanese-pack or C:\\packs\\japanese-pack\\pack.json"
              @blur="hydrateLocalDraftFromPath(localDraft.path)"
            />
            <button class="inline-action-btn" type="button" @click="browseLocalPackPath()">
              <Search :size="14" />
              Browse
            </button>
          </div>
        </label>
      </div>
      <label v-if="addMode === 'github'" class="toggle-row">
        <span>Enabled</span>
        <input v-model="remoteDraft.enabled" type="checkbox" />
      </label>
      <label v-else class="toggle-row">
        <span>Enabled</span>
        <input v-model="localDraft.enabled" type="checkbox" />
      </label>
      <div class="detail-actions">
        <button class="action-btn primary" @click="saveNewSource()">
          <Save :size="14" />
          {{ addMode === 'github' ? 'Save source' : 'Save local source' }}
        </button>
        <button class="action-btn" @click="closeAddModal()">
          Cancel
        </button>
      </div>
    </div>
  </OverlayShell>
</template>

<style scoped>
.packs-btn {
  position: fixed;
  top: 14px;
  right: 204px;
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

.packs-btn:hover,
.packs-btn.active {
  background: color-mix(in srgb, var(--app-accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 35%, transparent);
  color: var(--app-accent);
}

.packs-btn-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.packs-btn-key {
  font-size: 10px;
  font-family: ui-monospace, 'Cascadia Code', monospace;
  padding: 2px 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.pack-library-layout {
  display: grid;
  grid-template-columns: minmax(280px, 320px) minmax(0, 1fr);
  gap: 18px;
  padding: 16px;
  min-height: 100%;
}

.sidebar,
.detail-card,
.detail-empty {
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.035);
}

.sidebar {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.sidebar-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.section-switch {
  display: inline-flex;
  gap: 6px;
  padding: 4px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.04);
}

.switch-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 8px 12px;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--app-text-secondary);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.switch-btn span {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  font-size: 10px;
}

.switch-btn.active {
  background: color-mix(in srgb, var(--app-accent) 16%, transparent);
  color: var(--app-text-primary);
}

.sidebar-actions {
  display: inline-flex;
  gap: 8px;
}

.list-wrap {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  overflow: auto;
}

.sidebar-section-label {
  margin: 2px 2px 4px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.list-item {
  width: 100%;
  text-align: left;
  padding: 12px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(255, 255, 255, 0.03);
  color: inherit;
  cursor: pointer;
  transition:
    background 0.12s ease,
    border-color 0.12s ease,
    transform 0.12s ease;
}

.list-item:hover,
.list-item.active {
  transform: translateX(1px);
  border-color: color-mix(in srgb, var(--app-accent) 34%, transparent);
  background: color-mix(in srgb, var(--app-accent) 10%, transparent);
}

.list-item-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.list-item-head strong {
  font-size: 13px;
  color: var(--app-text-primary);
}

.item-badge {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 3px 8px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  color: var(--app-text-secondary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.list-item-sub,
.list-item-meta,
.detail-hero p,
.card-head span,
.field span,
.toggle-row span,
.meta-row span,
.detail-empty span {
  font-size: 12px;
  color: var(--app-text-secondary);
}

.list-item-sub,
.list-item-meta {
  margin-top: 5px;
  word-break: break-all;
}

.empty-state {
  padding: 18px 12px;
  color: var(--app-text-secondary);
  font-size: 12px;
}

.detail-pane {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.detail-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
}

.detail-kicker {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.detail-hero h2 {
  margin: 6px 0 0;
  font-size: 24px;
  color: var(--app-text-primary);
}

.detail-hero p {
  margin: 6px 0 0;
}

.detail-hero-actions,
.detail-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.detail-grid-single {
  grid-template-columns: 1fr;
}

.detail-card,
.detail-empty {
  padding: 16px;
}

.detail-empty {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.card-head {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
}

.card-head strong,
.detail-empty strong {
  font-size: 15px;
  color: var(--app-text-primary);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.field-span-2 {
  grid-column: 1 / -1;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.field-with-icon {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field-with-action {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field-with-icon svg {
  color: var(--app-text-secondary);
  flex-shrink: 0;
}

.field-with-icon input {
  flex: 1;
}

.field-with-action svg {
  color: var(--app-text-secondary);
  flex-shrink: 0;
}

.field-with-action input {
  flex: 1;
}

.inline-action-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.04);
  color: var(--app-text-primary);
  font-size: 12px;
  cursor: pointer;
}

.toggle-row,
.meta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.toggle-row {
  margin-top: 12px;
}

.meta-stack {
  display: grid;
  gap: 10px;
}

.meta-row {
  padding: 10px 0;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.meta-row:first-child {
  border-top: none;
  padding-top: 0;
}

.meta-row strong {
  font-size: 12px;
  color: var(--app-text-primary);
  text-align: right;
}

.action-btn,
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 8px 12px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.04);
  color: var(--app-text-primary);
  font-size: 12px;
  cursor: pointer;
}

.icon-btn {
  width: 34px;
  height: 34px;
  padding: 0;
}

.action-btn.primary,
.icon-btn.accent {
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 28%, transparent);
}

.action-btn.danger {
  color: #f3a3a0;
  border-color: rgba(243, 154, 143, 0.24);
  background: rgba(243, 154, 143, 0.08);
}

.action-btn:disabled {
  opacity: 0.45;
  cursor: default;
}

.error-box {
  margin-top: 12px;
  border-radius: 10px;
  padding: 10px 12px;
  color: #f39a8f;
  background: rgba(243, 154, 143, 0.08);
  border: 1px solid rgba(243, 154, 143, 0.22);
  font-size: 12px;
}

.add-modal-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
}

@media (max-width: 1180px) {
  .packs-btn {
    right: 188px;
  }

  .packs-btn-label {
    display: none;
  }
}

@media (max-width: 1080px) {
  .pack-library-layout,
  .detail-grid,
  .form-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .packs-btn {
    right: 126px;
  }

  .detail-hero {
    flex-direction: column;
  }
}
</style>
