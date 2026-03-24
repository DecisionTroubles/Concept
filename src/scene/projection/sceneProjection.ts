import { computed, type ComputedRef } from 'vue'
import type { PositionedNode } from '@/composables/useForceLayout'
import type { ConnectionLayer } from '@/bindings'
import type { GraphFocusState } from '@/scene/model/focusState'
import type { SceneSnapshot } from '@/scene/model/sceneSnapshot'
import { projectFocusNodes } from '@/scene/projection/focusProjection'
import { projectVisualState } from '@/scene/projection/visualProjection'
import { projectEdges } from '@/scene/projection/edgeProjection'

interface SceneProjectionInput {
  positionedNodes: ComputedRef<PositionedNode[]>
  focusState: ComputedRef<GraphFocusState>
  selectedNodeId: ComputedRef<string | null>
  hoveredNodeId: ComputedRef<string | null>
  pinnedNodeIds: ComputedRef<string[]>
  activeConnectionLayerIds: ComputedRef<string[]>
  connectionLayers: ComputedRef<ConnectionLayer[]>
  hasConnectionLayers: ComputedRef<boolean>
  themeVars: ComputedRef<Record<string, string>>
  focusViewConfig: ComputedRef<{ rings: number; ringRadius: number; maxNeighbors: number }>
}

export function useSceneProjection(input: SceneProjectionInput) {
  return computed<SceneSnapshot>(() => {
    const focusProjection = projectFocusNodes(
      input.positionedNodes.value,
      input.focusState.value,
      input.focusViewConfig.value,
    )
    const activeNodeId =
      input.focusState.value.mode === 'solar'
        ? input.focusState.value.cursorNodeId
        : input.selectedNodeId.value
    const visuals = projectVisualState({
      nodes: focusProjection.nodes,
      activeNodeId,
      hoveredNodeId: input.hoveredNodeId.value,
      pinnedNodeIds: input.pinnedNodeIds.value,
      themeVars: input.themeVars.value,
    })
    const nodes = visuals.map(visual => ({
      ...visual.node,
      parentNodeId: visual.parentNodeId,
      radius: visual.radius,
      scale: visual.scale,
      color: visual.color,
      emissive: visual.emissive,
      emissiveIntensity: visual.emissiveIntensity,
      neighbor: visual.neighbor,
      hovered: visual.hovered,
      selected: visual.selected,
      pinned: visual.pinned,
      progressStatus: visual.progressStatus,
      labelPriority: visual.labelPriority,
    }))
    const edges = projectEdges(
      focusProjection.nodes,
      input.focusState.value,
      input.activeConnectionLayerIds.value,
      input.connectionLayers.value,
      input.hasConnectionLayers.value,
    )

    return {
      mode: focusProjection.mode,
      nodes,
      edges,
      activeNodeId,
      focusRootNodeId: input.focusState.value.mode === 'solar' ? input.focusState.value.rootNodeId : null,
      hoveredNodeId: input.hoveredNodeId.value,
      activeConnectionLayerIds: [...input.activeConnectionLayerIds.value],
      hasConnectionLayers: input.hasConnectionLayers.value,
    }
  })
}
