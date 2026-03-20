import * as THREE from 'three'
import type { PositionedNode } from '@/composables/useForceLayout'
import type { GraphFocusState } from '@/scene/model/focusState'
import { isFocusVisibleEdge } from '@/scene/projection/focusProjection'
import type { SceneSnapshotEdge } from '@/scene/model/sceneSnapshot'

function edgeColor(edgeType: string): string {
  switch (edgeType) {
    case 'Prerequisite':
      return '#5b8fff'
    case 'Semantic':
      return '#5a648c'
    case 'UserDefined':
      return '#f59e0b'
    default:
      return '#4a5068'
  }
}

function edgeLineWidth(edgeType: string): number {
  switch (edgeType) {
    case 'Prerequisite':
      return 2.5
    case 'Semantic':
      return 1.2
    default:
      return 1.8
  }
}

export function projectEdges(
  nodes: PositionedNode[],
  focusState: GraphFocusState,
  activeConnectionLayerIds: string[],
  hasConnectionLayers: boolean,
): SceneSnapshotEdge[] {
  const nodeMap = new Map<string, PositionedNode>(nodes.map(node => [node.id, node]))
  const selectedLayerSet = new Set(activeConnectionLayerIds)
  const result: SceneSnapshotEdge[] = []
  const seenEdgeIds = new Set<string>()
  const bundles = new Map<string, Array<SceneSnapshotEdge & { source: PositionedNode; target: PositionedNode; shape: 'straight' | 'arc'; arcHeight: number }>>()

  for (const node of nodes) {
    for (const conn of node.connections) {
      if (seenEdgeIds.has(conn.id)) continue
      seenEdgeIds.add(conn.id)
      if (!isFocusVisibleEdge(focusState, node.id, conn)) continue
      if (
        hasConnectionLayers &&
        (
          selectedLayerSet.size === 0 ||
          conn.connection_layer_ids.length === 0 ||
          !conn.connection_layer_ids.some(id => selectedLayerSet.has(id))
        )
      ) {
        continue
      }

      const target = nodeMap.get(conn.target_id)
      if (!target) continue
      const a = node.id < conn.target_id ? node.id : conn.target_id
      const b = node.id < conn.target_id ? conn.target_id : node.id
      const pairKey = `${a}::${b}`
      const row = {
        id: conn.id,
        sourceId: node.id,
        targetId: conn.target_id,
        points: [] as [number, number, number][],
        color: edgeColor(conn.edge_type),
        width: edgeLineWidth(conn.edge_type),
        opacity: 0.9,
        dashed: false,
        dashSize: 0.22,
        gapSize: 0.14,
        dashScale: 1,
        source: node,
        target,
        shape: conn.edge_type === 'Semantic' ? ('arc' as const) : ('straight' as const),
        arcHeight: conn.edge_type === 'Semantic' ? 0.72 : 0,
      }
      const bucket = bundles.get(pairKey)
      if (bucket) bucket.push(row)
      else bundles.set(pairKey, [row])
    }
  }

  const up = new THREE.Vector3(0, 1, 0)
  const xAxis = new THREE.Vector3(1, 0, 0)
  const dir = new THREE.Vector3()
  const normal = new THREE.Vector3()
  const mid = new THREE.Vector3()

  for (const bundle of bundles.values()) {
    const count = bundle.length
    for (let i = 0; i < count; i += 1) {
      const row = bundle[i]
      const lane = i - (count - 1) / 2
      const offsetStrength = 0.38
      const src = row.source
      const tgt = row.target
      dir.set(tgt.x - src.x, tgt.y - src.y, tgt.z - src.z)
      if (dir.lengthSq() < 1e-6) continue
      dir.normalize()
      normal.crossVectors(dir, up)
      if (normal.lengthSq() < 1e-6) normal.crossVectors(dir, xAxis)
      normal.normalize()
      const totalOffset = row.arcHeight + (count > 1 && lane !== 0 ? offsetStrength * lane : 0)

      if (totalOffset !== 0) {
        normal.multiplyScalar(totalOffset)
        mid.set((src.x + tgt.x) / 2, (src.y + tgt.y) / 2, (src.z + tgt.z) / 2).add(normal)
        row.points = [
          [src.x, src.y, src.z],
          [mid.x, mid.y, mid.z],
          [tgt.x, tgt.y, tgt.z],
        ]
      } else {
        row.points = [
          [src.x, src.y, src.z],
          [tgt.x, tgt.y, tgt.z],
        ]
      }
      result.push({
        id: row.id,
        sourceId: row.sourceId,
        targetId: row.targetId,
        points: row.points,
        color: row.color,
        width: row.width,
        opacity: row.opacity,
        dashed: row.dashed,
        dashSize: row.dashSize,
        gapSize: row.gapSize,
        dashScale: row.dashScale,
      })
    }
  }

  return result
}
