import { computed, ref, type Ref } from 'vue'
import type {
  ConnectionLayer,
  Layer,
  Node,
  NodeProgress,
  NoteType,
  PackRegistryEntry,
  RelationKind,
  ReviewEvent,
  SchedulerDescriptor,
  WorldConfig,
  WorldPackInfo,
} from '@/bindings'
import type { GraphFocusState } from '@/scene/model/focusState'
import { createWorldFocusState, focusCursorNodeId, focusRootNodeId, isSolarFocusState } from '@/scene/model/focusState'

export type BufferId = 'none' | 'pinned' | 'packs' | 'map'

export interface GraphResourceState {
  layers: Ref<Layer[]>
  activeLayerId: Ref<string | null>
  nodes: Ref<Node[]>
  noteTypes: Ref<NoteType[]>
  nodeProgress: Ref<NodeProgress[]>
  schedulerAlgorithms: Ref<SchedulerDescriptor[]>
  reviewEvents: Ref<ReviewEvent[]>
  worldConfig: Ref<WorldConfig | null>
  worldPacks: Ref<WorldPackInfo[]>
  packRegistry: Ref<PackRegistryEntry[]>
  relationKinds: Ref<RelationKind[]>
  connectionLayers: Ref<ConnectionLayer[]>
  activeConnectionLayerIds: Ref<string[]>
  connectionLayerSelectionInitialized: Ref<boolean>
}

export function createGraphResourceState(): GraphResourceState {
  return {
    layers: ref<Layer[]>([]),
    activeLayerId: ref<string | null>(null),
    nodes: ref<Node[]>([]),
    noteTypes: ref<NoteType[]>([]),
    nodeProgress: ref<NodeProgress[]>([]),
    schedulerAlgorithms: ref<SchedulerDescriptor[]>([]),
    reviewEvents: ref<ReviewEvent[]>([]),
    worldConfig: ref<WorldConfig | null>(null),
    worldPacks: ref<WorldPackInfo[]>([]),
    packRegistry: ref<PackRegistryEntry[]>([]),
    relationKinds: ref<RelationKind[]>([]),
    connectionLayers: ref<ConnectionLayer[]>([]),
    activeConnectionLayerIds: ref<string[]>([]),
    connectionLayerSelectionInitialized: ref(false),
  }
}

export interface GraphSessionState {
  selectedNodeId: Ref<string | null>
  centeredNodePanel: Ref<boolean>
  nodeEditorOpen: Ref<boolean>
  pinnedNodeIds: Ref<string[]>
  focusState: Ref<GraphFocusState>
  focusViewActive: Ref<boolean>
  focusRootNodeId: Ref<string | null>
  focusCursorNodeId: Ref<string | null>
  focusOverlayParentSelection: Ref<string[] | null>
  activeBuffer: Ref<BufferId>
  packLibraryOpen: Ref<boolean>
  progressOverlayOpen: Ref<boolean>
  worldPickerOpen: Ref<boolean>
  focusVersion: Ref<number>
  focusViewVersion: Ref<number>
}

export function createGraphSessionState(): GraphSessionState {
  const focusState = ref<GraphFocusState>(createWorldFocusState())
  return {
    selectedNodeId: ref<string | null>(null),
    centeredNodePanel: ref(false),
    nodeEditorOpen: ref(false),
    pinnedNodeIds: ref<string[]>([]),
    focusState,
    focusViewActive: computed(() => isSolarFocusState(focusState.value)),
    focusRootNodeId: computed(() => focusRootNodeId(focusState.value)),
    focusCursorNodeId: computed(() => focusCursorNodeId(focusState.value)),
    focusOverlayParentSelection: ref<string[] | null>(null),
    activeBuffer: ref<BufferId>('none'),
    packLibraryOpen: ref(false),
    progressOverlayOpen: ref(false),
    worldPickerOpen: ref(false),
    focusVersion: ref(0),
    focusViewVersion: ref(0),
  }
}

export interface GraphStatusState {
  isLoading: Ref<boolean>
  error: Ref<string | null>
}

export function createGraphStatusState(): GraphStatusState {
  return {
    isLoading: ref(false),
    error: ref<string | null>(null),
  }
}

export function createGraphDerivedState(
  nodes: Ref<Node[]>,
  selectedNodeId: Ref<string | null>,
  pinnedNodeIds: Ref<string[]>,
) {
  const selectedNode = computed(() =>
    selectedNodeId.value ? (nodes.value.find(n => n.id === selectedNodeId.value) ?? null) : null
  )

  const pinnedNodes = computed(() => {
    const set = new Set(pinnedNodeIds.value)
    return nodes.value.filter(n => set.has(n.id))
  })

  const dueNodes = computed(() =>
    nodes.value.filter(n => {
      if (n.progress_status === 'mastered') {
        return n.progress_next_review_at ? Number(n.progress_next_review_at) <= Date.now() / 1000 : false
      }
      if (n.progress_status === 'new') return true
      if (!n.progress_next_review_at) return true
      return Number(n.progress_next_review_at) <= Date.now() / 1000
    })
  )

  return { selectedNode, pinnedNodes, dueNodes }
}
