<script setup lang="ts">
import { computed, ref } from 'vue'
import { Download, FolderOpen, Github, Package, Plus, RefreshCcw, Trash2 } from 'lucide-vue-next'

const graphStore = useGraphStore()

const showAddSource = ref(false)
const packForm = ref({
  id: '',
  name: '',
  repo: '',
  path: '',
  branch: 'main',
  pinned_ref: '',
  enabled: true,
})

const remoteProjects = computed(() =>
  [...graphStore.packRegistry].sort((a, b) => a.source.name.localeCompare(b.source.name))
)

const localProjects = computed(() =>
  graphStore.worldPacks
    .filter(world => world.source_kind === 'local')
    .sort((a, b) => (a.world_name ?? a.pack_path).localeCompare(b.world_name ?? b.pack_path))
)

const installedProjects = computed(() =>
  graphStore.worldPacks
    .filter(world => world.source_kind === 'installed')
    .sort((a, b) => (a.world_name ?? a.pack_path).localeCompare(b.world_name ?? b.pack_path))
)

function resetPackForm() {
  packForm.value = {
    id: '',
    name: '',
    repo: '',
    path: '',
    branch: 'main',
    pinned_ref: '',
    enabled: true,
  }
}

async function submitPackSource() {
  await graphStore.addGitHubPackSource({
    id: packForm.value.id.trim(),
    name: packForm.value.name.trim(),
    repo: packForm.value.repo.trim(),
    path: packForm.value.path.trim(),
    branch: packForm.value.branch.trim(),
    pinned_ref: packForm.value.pinned_ref.trim() || null,
    enabled: packForm.value.enabled,
  })
  resetPackForm()
  showAddSource.value = false
}

async function openWorld(worldId: string | null | undefined) {
  if (!worldId) return
  await graphStore.selectWorld(worldId)
}
</script>

<template>
  <div class="pack-library">
    <section class="library-hero">
      <div class="library-copy">
        <div class="library-eyebrow">Pack library</div>
        <h2>Projects and sources</h2>
        <p>
          Keep the source list here, then reload or pull changes when you want them locally.
        </p>
      </div>
      <div class="library-actions">
        <button class="action-btn" @click="graphStore.refreshPackRegistry()">
          <RefreshCcw :size="14" />
          Reload library
        </button>
        <button class="action-btn primary" @click="showAddSource = !showAddSource">
          <Plus :size="14" />
          {{ showAddSource ? 'Hide source form' : 'Add GitHub source' }}
        </button>
      </div>
    </section>

    <section v-if="showAddSource" class="library-add-card">
      <div class="section-head">
        <div>
          <div class="section-kicker">New source</div>
          <strong>Add a repo folder without cluttering the main list</strong>
        </div>
      </div>
      <div class="form-grid">
        <label class="field">
          <span>Pack id</span>
          <input v-model="packForm.id" type="text" placeholder="language-core" />
        </label>
        <label class="field">
          <span>Display name</span>
          <input v-model="packForm.name" type="text" placeholder="Language Core" />
        </label>
        <label class="field">
          <span>GitHub repo</span>
          <input v-model="packForm.repo" type="text" placeholder="owner/repo" />
        </label>
        <label class="field">
          <span>Folder path</span>
          <input v-model="packForm.path" type="text" placeholder="packs/language-core" />
        </label>
        <label class="field">
          <span>Branch</span>
          <input v-model="packForm.branch" type="text" placeholder="main" />
        </label>
        <label class="field">
          <span>Pinned ref</span>
          <input v-model="packForm.pinned_ref" type="text" placeholder="optional tag or sha" />
        </label>
      </div>
      <div class="form-actions">
        <button class="action-btn primary" @click="submitPackSource">Save source</button>
        <button class="action-btn" @click="resetPackForm(); showAddSource = false">Cancel</button>
      </div>
    </section>

    <section class="library-section">
      <div class="section-head">
        <div>
          <div class="section-kicker">Remote sources</div>
          <strong>{{ remoteProjects.length }} tracked project{{ remoteProjects.length === 1 ? '' : 's' }}</strong>
        </div>
        <span class="section-note">GitHub now, room for more providers later.</span>
      </div>

      <div v-if="remoteProjects.length === 0" class="empty-card">
        No remote projects yet. Add a source when you want this library to start tracking external packs.
      </div>

      <div v-else class="project-list">
        <article v-for="entry in remoteProjects" :key="entry.source.id" class="project-card">
          <div class="project-head">
            <div class="project-title">
              <strong>{{ entry.source.name }}</strong>
              <span>{{ entry.source.id }}</span>
            </div>
            <div class="project-badges">
              <span class="badge provider">
                <Github :size="12" />
                GitHub
              </span>
              <span class="badge status">{{ entry.install_status }}</span>
              <span v-if="entry.pack_info?.world_name" class="badge world">{{ entry.pack_info.world_name }}</span>
            </div>
          </div>

          <p class="project-path">{{ entry.source.repo }}/{{ entry.source.path }}</p>
          <p class="project-meta">
            Branch {{ entry.source.branch }}<template v-if="entry.source.pinned_ref"> · {{ entry.source.pinned_ref }}</template>
          </p>
          <p v-if="entry.last_error" class="project-error">{{ entry.last_error }}</p>

          <div class="project-actions">
            <button class="action-btn primary" @click="graphStore.installPackSource(entry.source.id)">
              <Download :size="13" />
              Pull
            </button>
            <button class="action-btn" @click="graphStore.refreshPackSource(entry.source.id)">
              <RefreshCcw :size="13" />
              Reload
            </button>
            <button class="action-btn" @click="graphStore.checkPackSourceUpdates(entry.source.id)">
              Check changes
            </button>
            <button
              v-if="entry.pack_info?.world_id"
              class="action-btn"
              @click="openWorld(entry.pack_info.world_id)"
            >
              <FolderOpen :size="13" />
              Open
            </button>
            <button class="action-btn danger" @click="graphStore.removePackSource(entry.source.id)">
              <Trash2 :size="13" />
              Remove
            </button>
          </div>
        </article>
      </div>
    </section>

    <section class="library-section">
      <div class="section-head">
        <div>
          <div class="section-kicker">Local packs</div>
          <strong>{{ localProjects.length }} local project{{ localProjects.length === 1 ? '' : 's' }}</strong>
        </div>
        <span class="section-note">Directly discovered from your local pack folders.</span>
      </div>

      <div v-if="localProjects.length === 0" class="empty-card">
        No local packs detected in app data.
      </div>

      <div v-else class="project-list compact">
        <article v-for="world in localProjects" :key="world.pack_path" class="project-card compact">
          <div class="project-head">
            <div class="project-title">
              <strong>{{ world.world_name ?? 'Invalid pack' }}</strong>
              <span>{{ world.world_id ?? world.pack_path }}</span>
            </div>
            <div class="project-badges">
              <span class="badge provider">
                <Package :size="12" />
                Local
              </span>
              <span v-if="world.is_loaded" class="badge world">Loaded</span>
              <span v-else-if="world.is_active" class="badge world">Selected</span>
            </div>
          </div>
          <p class="project-path">{{ world.pack_path }}</p>
          <p v-if="world.error" class="project-error">{{ world.error }}</p>
          <div class="project-actions">
            <button
              class="action-btn"
              :disabled="!world.valid || !world.world_id || world.is_active || graphStore.isLoading"
              @click="openWorld(world.world_id)"
            >
              <FolderOpen :size="13" />
              {{ world.is_active ? 'Current' : 'Open' }}
            </button>
          </div>
        </article>
      </div>
    </section>

    <section class="library-section">
      <div class="section-head">
        <div>
          <div class="section-kicker">Installed copies</div>
          <strong>{{ installedProjects.length }} installed pack{{ installedProjects.length === 1 ? '' : 's' }}</strong>
        </div>
        <button class="action-btn" @click="graphStore.reloadActiveWorld()">Reload active project</button>
      </div>

      <div v-if="installedProjects.length === 0" class="empty-card">
        Nothing pulled yet. Remote sources land here after you pull them.
      </div>

      <div v-else class="project-list compact">
        <article v-for="world in installedProjects" :key="world.pack_path" class="project-card compact">
          <div class="project-head">
            <div class="project-title">
              <strong>{{ world.world_name ?? 'Invalid pack' }}</strong>
              <span>{{ world.world_id ?? world.pack_path }}</span>
            </div>
            <div class="project-badges">
              <span class="badge provider">
                <Package :size="12" />
                Installed
              </span>
              <span v-if="world.is_loaded" class="badge world">Loaded</span>
              <span v-else-if="world.is_active" class="badge world">Selected</span>
            </div>
          </div>
          <p class="project-path">{{ world.pack_path }}</p>
          <p v-if="world.error" class="project-error">{{ world.error }}</p>
          <div class="project-actions">
            <button
              class="action-btn"
              :disabled="!world.valid || !world.world_id || world.is_active || graphStore.isLoading"
              @click="openWorld(world.world_id)"
            >
              <FolderOpen :size="13" />
              {{ world.is_active ? 'Current' : 'Open' }}
            </button>
          </div>
        </article>
      </div>
    </section>
  </div>
</template>

<style scoped>
.pack-library {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.library-hero,
.library-add-card,
.project-card,
.empty-card {
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.035);
}

.library-hero {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  padding: 16px;
}

.library-copy {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 520px;
}

.library-eyebrow,
.section-kicker {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.library-copy h2 {
  margin: 0;
  font-size: 24px;
  color: var(--app-text-primary);
}

.library-copy p,
.section-note,
.project-meta,
.project-path {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
  color: var(--app-text-secondary);
}

.library-actions,
.project-actions,
.form-actions,
.project-badges {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
}

.library-actions {
  align-self: flex-start;
  justify-content: flex-end;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 8px 12px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.04);
  color: var(--app-text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s, opacity 0.12s;
}

.action-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.18);
}

.action-btn.primary {
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

.library-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: end;
}

.section-head strong {
  font-size: 18px;
  color: var(--app-text-primary);
}

.library-add-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field span {
  font-size: 12px;
  color: var(--app-text-secondary);
}

.field input {
  width: 100%;
  min-width: 0;
}

.project-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.project-list.compact {
  grid-template-columns: 1fr;
}

.project-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
}

.project-card.compact {
  gap: 10px;
}

.project-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.project-title {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.project-title strong {
  font-size: 15px;
  color: var(--app-text-primary);
}

.project-title span {
  font-size: 12px;
  color: var(--app-text-secondary);
  word-break: break-all;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  border: 1px solid transparent;
}

.badge.provider,
.badge.status {
  color: var(--app-text-secondary);
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.08);
}

.badge.world {
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 30%, transparent);
}

.project-error {
  margin: 0;
  color: #f39a8f;
  font-size: 12px;
}

.empty-card {
  padding: 16px;
  color: var(--app-text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

@media (max-width: 980px) {
  .project-list,
  .form-grid {
    grid-template-columns: 1fr;
  }

  .library-hero,
  .section-head {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
