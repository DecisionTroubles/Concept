import * as THREE from 'three'
import { COMPASS_RING_R } from '@/scene/controller/useSceneHudState'
import type { SceneSnapshot } from '@/scene/model/sceneSnapshot'
import type { CompassHudModel } from '@/scene/model/hudModel'

function toScreenPosition(position: Pick<THREE.Vector3, 'x' | 'y' | 'z'>, camera: THREE.PerspectiveCamera) {
  const projected = new THREE.Vector3(position.x, position.y, position.z).project(camera)
  return {
    x: ((projected.x + 1) / 2) * window.innerWidth,
    y: ((-projected.y + 1) / 2) * window.innerHeight,
  }
}

function relationEdgeType(snapshot: SceneSnapshot, sourceId: string, targetId: string): string {
  const source = snapshot.nodes.find(node => node.id === sourceId)
  const forward = source?.connections.find(conn => conn.target_id === targetId)
  if (forward) return forward.edge_type
  const target = snapshot.nodes.find(node => node.id === targetId)
  const reverse = target?.connections.find(conn => conn.target_id === sourceId)
  if (reverse) return reverse.edge_type
  return 'Context'
}

function isConnectionVisible(snapshot: SceneSnapshot, connectionLayerIds: string[]): boolean {
  if (!snapshot.hasConnectionLayers) return true
  if (snapshot.activeConnectionLayerIds.length === 0) return false
  if (connectionLayerIds.length === 0) return false
  return connectionLayerIds.some(id => snapshot.activeConnectionLayerIds.includes(id))
}

function hasVisibleRelation(snapshot: SceneSnapshot, sourceId: string, targetId: string): boolean {
  const source = snapshot.nodes.find(node => node.id === sourceId)
  const forward = source?.connections.find(conn => conn.target_id === targetId)
  if (forward && isConnectionVisible(snapshot, forward.connection_layer_ids)) return true
  const target = snapshot.nodes.find(node => node.id === targetId)
  const reverse = target?.connections.find(conn => conn.target_id === sourceId)
  return Boolean(reverse && isConnectionVisible(snapshot, reverse.connection_layer_ids))
}

export function projectHud(snapshot: SceneSnapshot, camera: THREE.PerspectiveCamera): CompassHudModel {
  if (!snapshot.activeNodeId) {
    return { dots: [], center: null, neighborOrder: [], activeIndex: 0 }
  }

  if (snapshot.mode === 'solar' && snapshot.focusRootNodeId) {
    const root = snapshot.nodes.find(node => node.id === snapshot.focusRootNodeId)
    if (!root) {
      return { dots: [], center: null, neighborOrder: [], activeIndex: 0 }
    }

    const center = toScreenPosition(root, camera)
    const neighbors = snapshot.nodes
      .filter(node => node.id !== root.id && node.parentNodeId === root.id && hasVisibleRelation(snapshot, root.id, node.id))
      .slice(0, 9)
      .map(node => {
        const screen = toScreenPosition(node, camera)
        return {
          id: node.id,
          title: node.title,
          edgeType: relationEdgeType(snapshot, root.id, node.id),
          angle: Math.atan2(screen.y - center.y, screen.x - center.x),
          screenX: screen.x,
          screenY: screen.y,
        }
      })
      .sort((a, b) => a.angle - b.angle)

    if (neighbors.length === 0) {
      const worldLikeSnapshot: SceneSnapshot = {
        ...snapshot,
        mode: 'world',
        activeNodeId: root.id,
      }
      return projectHud(worldLikeSnapshot, camera)
    }

    const dots = neighbors.map((item, idx) => {
      let ring = COMPASS_RING_R
      if (idx > 0) {
        const previous = neighbors[idx - 1]
        if (previous) {
          const gap = Math.abs(item.angle - previous.angle)
          if (gap < 0.23) ring += 16 + (0.23 - gap) * 55
        }
      }
      return {
        id: item.id,
        title: item.title,
        screenX: center.x + Math.cos(item.angle) * ring,
        screenY: center.y + Math.sin(item.angle) * ring,
        edgeType: item.edgeType,
        index: idx + 1,
      }
    })

    const activeIndex = Math.max(0, neighbors.findIndex(node => node.id === snapshot.activeNodeId))
    return {
      dots,
      center,
      neighborOrder: neighbors.map(node => node.id),
      activeIndex,
    }
  }

  const selected = snapshot.nodes.find(node => node.id === snapshot.activeNodeId)
  if (!selected) {
    return { dots: [], center: null, neighborOrder: [], activeIndex: 0 }
  }
  const nodeMap = new Map(snapshot.nodes.map(node => [node.id, node]))
  const center = toScreenPosition(selected, camera)
  const seen = new Set<string>()
  const validConns = selected.connections
    .filter(conn => {
      if (!isConnectionVisible(snapshot, conn.connection_layer_ids)) return false
      if (!nodeMap.has(conn.target_id) || seen.has(conn.target_id)) return false
      seen.add(conn.target_id)
      return true
    })
  const provisional = validConns.slice(0, 9).map((conn, index) => {
    const node = nodeMap.get(conn.target_id)!
    const screen = toScreenPosition(node, camera)
    return {
      conn,
      index,
      title: node.title,
      angle: Math.atan2(screen.y - center.y, screen.x - center.x),
    }
  }).sort((a, b) => a.angle - b.angle)

  const dots = provisional.map((item, idx) => {
    let ring = COMPASS_RING_R
    if (idx > 0) {
      const previous = provisional[idx - 1]
      if (previous) {
        const gap = Math.abs(item.angle - previous.angle)
        if (gap < 0.23) ring += 16 + (0.23 - gap) * 55
      }
    }
    return {
      id: item.conn.target_id,
      title: item.title,
      screenX: center.x + Math.cos(item.angle) * ring,
      screenY: center.y + Math.sin(item.angle) * ring,
      edgeType: item.conn.edge_type,
      index: item.index + 1,
    }
  })

  const activeIndex = Math.max(0, validConns.findIndex(conn => conn.target_id === snapshot.activeNodeId))
  return {
    dots,
    center,
    neighborOrder: validConns.map(conn => conn.target_id),
    activeIndex,
  }
}
