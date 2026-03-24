<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useForceLayout } from '@/composables/useForceLayout'
import { useTheme } from '@/composables/useTheme'
import GraphRenderer from '@/scene/renderer/GraphRenderer.vue'
import { useSceneProjection } from '@/scene/projection/sceneProjection'
import { useCameraController } from '@/scene/controller/useCameraController'
import { useSceneController } from '@/scene/controller/useSceneController'
import { useSceneInputRouter } from '@/scene/controller/useSceneInputRouter'
import { useSceneHudState } from '@/scene/controller/useSceneHudState'
import { graphTrace } from '@/stores/graph/debug'

const graphStore = useGraphStore()
const editorMode = useEditorMode()
const themeState = useTheme()
const hudState = useSceneHudState()

const hoveredNodeId = ref<string | null>(null)

type JsonObject = Record<string, unknown>

function parseJsonObject(raw: string | null | undefined): JsonObject {
  if (!raw) return {}
  try {
    const parsed = JSON.parse(raw)
    return parsed && typeof parsed === 'object' ? (parsed as JsonObject) : {}
  } catch {
    return {}
  }
}

function groupsFromTags(tags: string[]): string[] {
  const out: string[] = []
  for (const tag of tags) {
    const lower = tag.toLowerCase()
    if (lower.startsWith('group:')) out.push(tag.slice(6))
    else if (lower.startsWith('cluster:')) out.push(tag.slice(8))
  }
  return Array.from(new Set(out))
}

function numOr(obj: JsonObject, key: string, fallback: number): number {
  const value = obj[key]
  return typeof value === 'number' ? value : fallback
}

const worldRoot = computed(() => parseJsonObject(graphStore.worldConfig?.config_json))
const worldMetadata = computed<JsonObject>(() => {
  const metadata = worldRoot.value.metadata
  return metadata && typeof metadata === 'object' ? (metadata as JsonObject) : {}
})

const focusViewConfig = computed(() => {
  const raw = worldMetadata.value.focus_view
  const obj = raw && typeof raw === 'object' ? (raw as JsonObject) : {}
  return {
    rings: Math.max(1, Math.min(3, Math.round(numOr(obj, 'rings', 1)))),
    ringRadius: Math.max(5, numOr(obj, 'ring_radius', 8.5)),
    maxNeighbors: Math.max(6, Math.min(48, Math.round(numOr(obj, 'max_neighbors', 18)))),
  }
})

const layoutClusterOptions = computed(() => {
  const membershipsByNodeId: Record<string, string[]> = {}
  for (const node of graphStore.nodes) {
    const groups = groupsFromTags(node.tags)
    if (groups.length) membershipsByNodeId[node.id] = groups
  }
  if (Object.keys(membershipsByNodeId).length === 0) return undefined
  return {
    cluster: {
      membershipsByNodeId,
      groupCohesionById: {},
      groupIntraSpacingById: {},
      interGroupSpacing: numOr(worldMetadata.value, 'inter_group_spacing', 24),
    },
  }
})

const { positionedNodes } = useForceLayout(
  computed(() => graphStore.nodes),
  computed(() => layoutClusterOptions.value),
  settled => {
    for (const node of settled) {
      graphStore.updateNodePosition(node.id, node.x, node.y, node.z)
    }
  }
)

const snapshot = useSceneProjection({
  positionedNodes: computed(() => positionedNodes.value),
  focusState: computed(() => graphStore.focusState),
  selectedNodeId: computed(() => graphStore.selectedNodeId),
  hoveredNodeId: computed(() => hoveredNodeId.value),
  pinnedNodeIds: computed(() => graphStore.pinnedNodeIds),
  activeConnectionLayerIds: computed(() => graphStore.activeConnectionLayerIds),
  connectionLayers: computed(() => graphStore.connectionLayers),
  hasConnectionLayers: computed(() => graphStore.connectionLayers.length > 0),
  themeVars: computed(() => themeState.activeTheme.value?.vars ?? {}),
  focusViewConfig,
})

const cameraController = useCameraController()
const sceneController = useSceneController({
  snapshot,
  cameraController,
})

const inputRouter = useSceneInputRouter({
  controller: sceneController,
})

watch(
  () => graphStore.focusVersion,
  () => {
    graphTrace('scene.watch.focusVersion', {
      focusVersion: graphStore.focusVersion,
      activeNodeId: snapshot.value.activeNodeId,
    })
    sceneController.requestNodeFocus(snapshot.value.activeNodeId)
  },
  { flush: 'post' }
)

watch(
  () => graphStore.focusViewVersion,
  () => {
    graphTrace('scene.watch.focusViewVersion', {
      focusViewVersion: graphStore.focusViewVersion,
      mode: snapshot.value.mode,
      nodeCount: snapshot.value.nodes.length,
    })
    sceneController.requestNodeFocus(snapshot.value.activeNodeId)
  },
  { flush: 'post' }
)

watch(
  () => graphStore.selectedNodeId,
  selectedNodeId => {
    hudState.setActiveNode(selectedNodeId)
  },
  { flush: 'post' }
)

watch(
  () => editorMode.mode.value,
  mode => {
    if (mode !== 'graph') hudState.clearHud()
  },
  { flush: 'post' }
)

function handleNodeClicked(nodeId: string) {
  sceneController.clickNode(nodeId)
}

function handleNodeHovered(nodeId: string | null) {
  hoveredNodeId.value = nodeId
  sceneController.hoverNode(nodeId)
}

</script>

<template>
  <GraphRenderer
    :snapshot="snapshot"
    :camera-controller="cameraController"
    :active-keys="inputRouter.activeKeys"
    @node-clicked="handleNodeClicked"
    @node-hovered="handleNodeHovered"
  />
</template>
