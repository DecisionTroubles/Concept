import * as THREE from 'three'
import type { PositionedNode } from '@/composables/useForceLayout'
import type { GraphFocusState } from '@/scene/model/focusState'

export interface FocusViewConfig {
  rings: number
  ringRadius: number
  maxNeighbors: number
}

export function isSublayerNode(node: { parent_node_id: string | null }): boolean {
  return typeof node.parent_node_id === 'string' && node.parent_node_id.length > 0
}

export function focusParentId(node: { parent_node_id: string | null } | null | undefined): string | null {
  return node?.parent_node_id ?? null
}

function snapshotConnections(node: PositionedNode): PositionedNode['connections'] {
  return node.connections.map(conn => ({
    ...conn,
    connection_layer_ids: [...conn.connection_layer_ids],
  }))
}

export function snapshotNode(node: PositionedNode, position?: Pick<PositionedNode, 'x' | 'y' | 'z'>): PositionedNode {
  return {
    ...node,
    note_fields: { ...node.note_fields },
    tags: [...node.tags],
    connections: snapshotConnections(node),
    x: position?.x ?? node.x,
    y: position?.y ?? node.y,
    z: position?.z ?? node.z,
  }
}

export function projectFocusNodes(
  positionedNodes: PositionedNode[],
  focusState: GraphFocusState,
  config: FocusViewConfig,
) {
  const worldNodes = positionedNodes.filter(node => !isSublayerNode(node))
  if (focusState.mode !== 'solar') {
    return {
      mode: 'world' as const,
      nodes: worldNodes.map(node => snapshotNode(node)),
    }
  }

  const root = worldNodes.find(node => node.id === focusState.rootNodeId)
  if (!root) {
    return {
      mode: 'world' as const,
      nodes: worldNodes.map(node => snapshotNode(node)),
    }
  }

  const focusChildren = positionedNodes
    .filter(node => focusParentId(node) === root.id)
    .slice(0, config.maxNeighbors)

  const buckets: PositionedNode[][] = Array.from(
    { length: Math.max(1, Math.min(config.rings, Math.max(focusChildren.length, 1))) },
    () => []
  )
  focusChildren.forEach((neighbor, index) => {
    buckets[index % buckets.length].push(neighbor)
  })

  const result: PositionedNode[] = [snapshotNode(root)]
  const rootBase = new THREE.Vector3(root.x, root.y, root.z)
  const baseVec = new THREE.Vector3()
  const fallbackAngleStep = Math.PI * (3 - Math.sqrt(5))

  buckets
    .filter(bucket => bucket.length > 0)
    .forEach((ringNodes, ringIndex) => {
      const radius = config.ringRadius * (ringIndex + 1)
      const items = [...ringNodes]
      items.sort((a, b) => {
        baseVec.set(a.x - rootBase.x, 0, a.z - rootBase.z)
        const angleA = baseVec.lengthSq() > 0.0001 ? Math.atan2(baseVec.z, baseVec.x) : ringIndex + fallbackAngleStep
        baseVec.set(b.x - rootBase.x, 0, b.z - rootBase.z)
        const angleB = baseVec.lengthSq() > 0.0001 ? Math.atan2(baseVec.z, baseVec.x) : ringIndex + fallbackAngleStep
        return angleA - angleB
      })

      const step = (2 * Math.PI) / Math.max(items.length, 1)
      items.forEach((node, itemIndex) => {
        const angle = items.length === 1 ? -Math.PI / 2 : -Math.PI / 2 + itemIndex * step
        result.push(snapshotNode(node, {
          x: root.x + Math.cos(angle) * radius,
          y: root.y + (ringIndex === 0 ? 0.8 : 0.4) + Math.sin(angle * 2) * 0.9,
          z: root.z + Math.sin(angle) * radius,
        }))
      })
    })

  return {
    mode: 'solar' as const,
    nodes: result,
  }
}

export function isFocusVisibleEdge(focusState: GraphFocusState, nodeId: string, conn: { target_id: string }): boolean {
  if (focusState.mode !== 'solar') return true
  return nodeId === focusState.rootNodeId || conn.target_id === focusState.rootNodeId
}
