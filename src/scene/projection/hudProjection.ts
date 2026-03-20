import * as THREE from 'three'
import { COMPASS_RING_R } from '@/scene/controller/useSceneHudState'
import type { SceneSnapshot } from '@/scene/model/sceneSnapshot'
import type { CompassHudModel } from '@/scene/model/hudModel'

export function projectHud(snapshot: SceneSnapshot, camera: THREE.PerspectiveCamera): CompassHudModel {
  if (!snapshot.activeNodeId) {
    return { dots: [], center: null, neighborOrder: [], activeIndex: 0 }
  }
  const selected = snapshot.nodes.find(node => node.id === snapshot.activeNodeId)
  if (!selected) {
    return { dots: [], center: null, neighborOrder: [], activeIndex: 0 }
  }
  const nodeMap = new Map(snapshot.nodes.map(node => [node.id, node]))
  const ndc = new THREE.Vector3(selected.x, selected.y, selected.z).project(camera)
  const sx = ((ndc.x + 1) / 2) * window.innerWidth
  const sy = ((-ndc.y + 1) / 2) * window.innerHeight
  const center = { x: sx, y: sy }
  const seen = new Set<string>()
  const validConns = selected.connections
    .filter(conn => {
      if (!nodeMap.has(conn.target_id) || seen.has(conn.target_id)) return false
      seen.add(conn.target_id)
      return true
    })
  const provisional = validConns.slice(0, 9).map((conn, index) => {
    const node = nodeMap.get(conn.target_id)!
    const vec = new THREE.Vector3(node.x, node.y, node.z).project(camera)
    const nx = ((vec.x + 1) / 2) * window.innerWidth
    const ny = ((-vec.y + 1) / 2) * window.innerHeight
    return {
      conn,
      index,
      title: node.title,
      angle: Math.atan2(ny - sy, nx - sx),
    }
  }).sort((a, b) => a.angle - b.angle)

  const dots = provisional.map((item, idx) => {
    let ring = COMPASS_RING_R
    if (idx > 0) {
      const gap = Math.abs(item.angle - provisional[idx - 1].angle)
      if (gap < 0.23) ring += 16 + (0.23 - gap) * 55
    }
    return {
      id: item.conn.target_id,
      title: item.title,
      screenX: sx + Math.cos(item.angle) * ring,
      screenY: sy + Math.sin(item.angle) * ring,
      edgeType: item.conn.edge_type,
      index: item.index + 1,
    }
  })

  return {
    dots,
    center,
    neighborOrder: validConns.map(conn => conn.target_id),
    activeIndex: 0,
  }
}
