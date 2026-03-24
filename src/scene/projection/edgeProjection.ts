import * as THREE from 'three'
import type { PositionedNode } from '@/composables/useForceLayout'
import type { ConnectionLayer } from '@/bindings'
import type { GraphFocusState } from '@/scene/model/focusState'
import { isFocusVisibleEdge } from '@/scene/projection/focusProjection'
import type { SceneSnapshotEdge } from '@/scene/model/sceneSnapshot'

function parseJson(raw: string | null | undefined): Record<string, unknown> {
  if (!raw) return {}
  try {
    const parsed = JSON.parse(raw)
    return parsed && typeof parsed === 'object' ? (parsed as Record<string, unknown>) : {}
  } catch {
    return {}
  }
}

function layerStyleMap(connectionLayers: ConnectionLayer[]): Map<string, Record<string, unknown>> {
  const map = new Map<string, Record<string, unknown>>()
  for (const layer of connectionLayers) {
    const md = parseJson(layer.metadata)
    const style = md.edge_style
    if (style && typeof style === 'object') map.set(layer.id, style as Record<string, unknown>)
  }
  return map
}

function firstLayerStyle(
  connectionLayerIds: string[],
  stylesByLayerId: Map<string, Record<string, unknown>>,
): Record<string, unknown> | null {
  for (const layerId of connectionLayerIds) {
    const style = stylesByLayerId.get(layerId)
    if (style) return style
  }
  return null
}

function edgeColor(edgeType: string, connectionLayerIds: string[], stylesByLayerId: Map<string, Record<string, unknown>>): string {
  const style = firstLayerStyle(connectionLayerIds, stylesByLayerId)
  if (typeof style?.color === 'string') return style.color

  switch (edgeType) {
    case 'Prerequisite':
      return '#60a5fa'
    case 'Semantic':
      return '#94a3b8'
    case 'UserDefined':
      return '#f59e0b'
    default:
      return '#64748b'
  }
}

function edgeLineWidth(
  edgeType: string,
  connectionLayerIds: string[],
  stylesByLayerId: Map<string, Record<string, unknown>>,
): number {
  const style = firstLayerStyle(connectionLayerIds, stylesByLayerId)
  if (typeof style?.width === 'number') return Math.max(2.2, style.width)

  switch (edgeType) {
    case 'Prerequisite':
      return 3.2
    case 'Semantic':
      return 2.2
    default:
      return 2.6
  }
}

function edgeDashConfig(
  connectionLayerIds: string[],
  stylesByLayerId: Map<string, Record<string, unknown>>,
): { dashed: boolean; dashSize: number; gapSize: number } {
  const style = firstLayerStyle(connectionLayerIds, stylesByLayerId)
  if (typeof style?.dash_size === 'number' && style.dash_size > 0) {
    return {
      dashed: true,
      dashSize: Math.max(0.1, style.dash_size),
      gapSize: Math.max(0.14, Math.min(0.48, Number(style.dash_size) * 0.7)),
    }
  }
  if (connectionLayerIds.includes('register')) {
    return { dashed: true, dashSize: 0.52, gapSize: 0.18 }
  }
  if (connectionLayerIds.includes('memory')) {
    return { dashed: true, dashSize: 0.14, gapSize: 0.38 }
  }
  return { dashed: false, dashSize: 0.22, gapSize: 0.14 }
}

function edgeShape(connectionLayerIds: string[], edgeType: string): { shape: 'straight' | 'arc'; arcHeight: number } {
  if (connectionLayerIds.includes('composition')) {
    return { shape: 'straight', arcHeight: 0 }
  }
  if (connectionLayerIds.includes('usage')) {
    return { shape: 'arc', arcHeight: 1.05 }
  }
  if (connectionLayerIds.includes('register')) {
    return { shape: 'arc', arcHeight: 1.9 }
  }
  if (connectionLayerIds.includes('memory')) {
    return { shape: 'arc', arcHeight: 2.8 }
  }
  return edgeType === 'Semantic'
    ? { shape: 'arc', arcHeight: 0.9 }
    : { shape: 'straight', arcHeight: 0 }
}

export function projectEdges(
  nodes: PositionedNode[],
  focusState: GraphFocusState,
  activeConnectionLayerIds: string[],
  connectionLayers: ConnectionLayer[],
  hasConnectionLayers: boolean,
): SceneSnapshotEdge[] {
  const nodeMap = new Map<string, PositionedNode>(nodes.map(node => [node.id, node]))
  const selectedLayerSet = new Set(activeConnectionLayerIds)
  const stylesByLayerId = layerStyleMap(connectionLayers)
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
      const shapeConfig = edgeShape(conn.connection_layer_ids, conn.edge_type)
      const a = node.id < conn.target_id ? node.id : conn.target_id
      const b = node.id < conn.target_id ? conn.target_id : node.id
      const pairKey = `${a}::${b}`
      const row = {
        id: conn.id,
        sourceId: node.id,
        targetId: conn.target_id,
        points: [] as [number, number, number][],
        color: edgeColor(conn.edge_type, conn.connection_layer_ids, stylesByLayerId),
        width: edgeLineWidth(conn.edge_type, conn.connection_layer_ids, stylesByLayerId),
        opacity: conn.connection_layer_ids.includes('composition') ? 1 : 0.98,
        dashed: edgeDashConfig(conn.connection_layer_ids, stylesByLayerId).dashed,
        dashSize: edgeDashConfig(conn.connection_layer_ids, stylesByLayerId).dashSize,
        gapSize: edgeDashConfig(conn.connection_layer_ids, stylesByLayerId).gapSize,
        dashScale: 1,
        source: node,
        target,
        shape: shapeConfig.shape,
        arcHeight: shapeConfig.arcHeight,
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
