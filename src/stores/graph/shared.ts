import { computed, ref, type Ref } from 'vue'
import type {
  ConnectionLayer,
  Layer,
  Node,
  NodeProgress,
  NoteType,
  RelationKind,
  ReviewEvent,
  SchedulerDescriptor,
  WorldConfig,
  WorldPackInfo,
} from '@/bindings'

export type BufferId = 'none' | 'pinned' | 'map'

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
  focusViewActive: Ref<boolean>
  focusRootNodeId: Ref<string | null>
  focusCursorNodeId: Ref<string | null>
  focusOverlayParentSelection: Ref<string[] | null>
  activeBuffer: Ref<BufferId>
  progressOverlayOpen: Ref<boolean>
  worldPickerOpen: Ref<boolean>
  focusVersion: Ref<number>
  focusViewVersion: Ref<number>
}

export function createGraphSessionState(): GraphSessionState {
  return {
    selectedNodeId: ref<string | null>(null),
    centeredNodePanel: ref(false),
    nodeEditorOpen: ref(false),
    pinnedNodeIds: ref<string[]>([]),
    focusViewActive: ref(false),
    focusRootNodeId: ref<string | null>(null),
    focusCursorNodeId: ref<string | null>(null),
    focusOverlayParentSelection: ref<string[] | null>(null),
    activeBuffer: ref<BufferId>('none'),
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
