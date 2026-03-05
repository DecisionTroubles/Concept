import { ref, watch, type Ref } from 'vue'
import type { Node } from '@/bindings'

export interface PositionedNode extends Node {
  x: number
  y: number
  z: number
}

interface SimNode {
  id: string
  x: number
  y: number
  z: number
  vx?: number
  vy?: number
  vz?: number
}

interface Vec3 {
  x: number
  y: number
  z: number
}

export interface ClusterLayoutOptions {
  membershipsByNodeId: Record<string, string[]>
  groupCohesionById?: Record<string, number>
  groupIntraSpacingById?: Record<string, number>
  interGroupSpacing?: number
}

export interface ForceLayoutOptions {
  cluster?: ClusterLayoutOptions
}

function randSeeded(seed: number): () => number {
  let s = seed >>> 0
  return () => {
    s = (1664525 * s + 1013904223) >>> 0
    return s / 4294967296
  }
}

function jitter3(seed: number, amount: number): Vec3 {
  const rnd = randSeeded(seed)
  return {
    x: (rnd() * 2 - 1) * amount,
    y: (rnd() * 2 - 1) * amount,
    z: (rnd() * 2 - 1) * amount,
  }
}

function buildGroupCenters(groupIds: string[], interGroupSpacing: number): Map<string, Vec3> {
  const centers = new Map<string, Vec3>()
  if (groupIds.length === 0) return centers
  const n = groupIds.length
  const ringRadius = Math.max(12, interGroupSpacing * (1.4 + Math.sqrt(n) * 0.55))
  for (let i = 0; i < n; i++) {
    const theta = (2 * Math.PI * i) / n
    const yWave = Math.sin(theta * 2.1) * Math.max(1.6, interGroupSpacing * 0.14)
    centers.set(groupIds[i], {
      x: ringRadius * Math.cos(theta),
      y: yWave,
      z: ringRadius * Math.sin(theta),
    })
  }
  return centers
}

function getNodeGroups(node: Node, cluster?: ClusterLayoutOptions): string[] {
  if (!cluster) return []
  return cluster.membershipsByNodeId[node.id] ?? []
}

function buildAnchorForNode(
  node: Node,
  centers: Map<string, Vec3>,
  cluster?: ClusterLayoutOptions,
): Vec3 | null {
  if (!cluster) return null
  const groups = getNodeGroups(node, cluster)
  if (groups.length === 0) return null

  let sumW = 0
  let x = 0
  let y = 0
  let z = 0
  for (const gid of groups) {
    const c = centers.get(gid)
    if (!c) continue
    const w = Math.max(0.2, cluster.groupCohesionById?.[gid] ?? 1)
    x += c.x * w
    y += c.y * w
    z += c.z * w
    sumW += w
  }
  if (sumW <= 0) return null
  return { x: x / sumW, y: y / sumW, z: z / sumW }
}

function sameGroup(nodeA: Node, nodeB: Node, cluster?: ClusterLayoutOptions): boolean {
  if (!cluster) return false
  const a = getNodeGroups(nodeA, cluster)
  const b = getNodeGroups(nodeB, cluster)
  if (a.length === 0 || b.length === 0) return false
  const set = new Set(a)
  return b.some(id => set.has(id))
}

// ---------------------------------------------------------------------------
// Instant layout — shown on the first render tick so nodes are always visible
// even while the force simulation is computing in the background.
// ---------------------------------------------------------------------------
function instantLayout(nodes: Node[], options?: ForceLayoutOptions): PositionedNode[] {
  const n = nodes.length
  if (n === 0) return []
  const cluster = options?.cluster
  const inter = cluster?.interGroupSpacing ?? 22
  const allGroupIds = cluster
    ? Array.from(new Set(Object.values(cluster.membershipsByNodeId).flat())).sort()
    : []
  const groupCenters = buildGroupCenters(allGroupIds, inter)
  const r = Math.max(8, n * 1.2)
  return nodes.map((node, i) => ({
    ...node,
    x: node.pos_x ?? (buildAnchorForNode(node, groupCenters, cluster)?.x ?? r * Math.cos((2 * Math.PI * i) / n)),
    y: node.pos_y ?? (buildAnchorForNode(node, groupCenters, cluster)?.y ?? 0),
    z: node.pos_z ?? (buildAnchorForNode(node, groupCenters, cluster)?.z ?? r * Math.sin((2 * Math.PI * i) / n)),
  }))
}

// ---------------------------------------------------------------------------
// Main composable
// ---------------------------------------------------------------------------
export function useForceLayout(
  nodes: Ref<Node[]>,
  options: Ref<ForceLayoutOptions | undefined>,
  onPositionsSettled?: (positioned: PositionedNode[]) => void,
) {
  const positionedNodes = ref<PositionedNode[]>([])

  watch(
    [nodes, options],
    async ([newNodes, newOptions]) => {
      if (newNodes.length === 0) {
        positionedNodes.value = []
        return
      }

      // --- Fast path: every node has a stored position from a previous run ---
      if (newNodes.every((n) => n.pos_x !== null && n.pos_y !== null && n.pos_z !== null)) {
        positionedNodes.value = newNodes.map((n) => ({
          ...n,
          x: n.pos_x!,
          y: n.pos_y!,
          z: n.pos_z!,
        }))
        return
      }

      // --- Show nodes immediately in a circle so the scene is never blank ---
      positionedNodes.value = instantLayout(newNodes, newOptions)

      // --- Run force layout in background, then update positions ---
      try {
        // Dynamic import keeps this chunk out of the critical boot path.
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const d3: any = await import('d3-force-3d')

        const count = newNodes.length
        const cluster = newOptions?.cluster
        const interGroupSpacing = Math.max(10, cluster?.interGroupSpacing ?? 24)
        const allGroupIds = cluster
          ? Array.from(new Set(Object.values(cluster.membershipsByNodeId).flat())).sort()
          : []
        const groupCenters = buildGroupCenters(allGroupIds, interGroupSpacing)

        // Golden-angle sphere distribution for diverse initial positions.
        const simNodes: SimNode[] = newNodes.map((node, i) => {
          const phi = Math.acos(1 - (2 * (i + 0.5)) / count)
          const theta = Math.PI * (1 + Math.sqrt(5)) * i
          const r = 12
          const anchor = buildAnchorForNode(node, groupCenters, cluster)
          const j = jitter3(i * 2654435761, 2.2)
          return {
            id: node.id,
            x: node.pos_x ?? ((anchor?.x ?? 0) + r * Math.sin(phi) * Math.cos(theta) * 0.35 + j.x),
            y: node.pos_y ?? ((anchor?.y ?? 0) + r * Math.cos(phi) * 0.2 + j.y * 0.5),
            z: node.pos_z ?? ((anchor?.z ?? 0) + r * Math.sin(phi) * Math.sin(theta) * 0.35 + j.z),
          }
        })

        const nodeIdSet = new Set(newNodes.map((n) => n.id))
        const nodeById = new Map<string, Node>(newNodes.map(n => [n.id, n]))
        const simLinks: { source: string; target: string }[] = []
        const seenPairs = new Set<string>()
        for (const node of newNodes) {
          for (const conn of node.connections) {
            if (nodeIdSet.has(conn.target_id)) {
              // Keep one physical spring per node pair for performance/stability.
              // Visual layer can still render multiple semantic relations separately.
              const a = node.id < conn.target_id ? node.id : conn.target_id
              const b = node.id < conn.target_id ? conn.target_id : node.id
              const pairKey = `${a}::${b}`
              if (seenPairs.has(pairKey)) continue
              seenPairs.add(pairKey)
              simLinks.push({ source: node.id, target: conn.target_id })
            }
          }
        }

        const intraDistanceByGroup = cluster?.groupIntraSpacingById ?? {}
        const linkDistance = (link: { source: SimNode | string; target: SimNode | string }) => {
          const sourceId = typeof link.source === 'string' ? link.source : link.source.id
          const targetId = typeof link.target === 'string' ? link.target : link.target.id
          const sourceNode = nodeById.get(sourceId)
          const targetNode = nodeById.get(targetId)
          if (!sourceNode || !targetNode) return 8
          if (sameGroup(sourceNode, targetNode, cluster)) {
            const sourceGroups = getNodeGroups(sourceNode, cluster)
            const targetGroups = getNodeGroups(targetNode, cluster)
            const shared = sourceGroups.find(g => targetGroups.includes(g))
            const base = shared ? intraDistanceByGroup[shared] : undefined
            return Math.max(4.2, (base ?? 6.2) * 0.92)
          }
          return Math.max(8.5, interGroupSpacing * 0.58)
        }

        const clusterForce = (alpha: number) => {
          if (!cluster) return
          for (const simNode of simNodes) {
            const src = nodeById.get(simNode.id)
            if (!src) continue
            const groups = getNodeGroups(src, cluster)
            if (groups.length === 0) continue
            const anchor = buildAnchorForNode(src, groupCenters, cluster)
            if (!anchor) continue
            let localCohesion = 1
            for (const gid of groups) {
              localCohesion = Math.max(localCohesion, cluster.groupCohesionById?.[gid] ?? 1)
            }
            const k = 0.06 * localCohesion * alpha
            simNode.vx = (simNode.vx ?? 0) + (anchor.x - simNode.x) * k
            simNode.vy = (simNode.vy ?? 0) + (anchor.y - simNode.y) * (k * 0.75)
            simNode.vz = (simNode.vz ?? 0) + (anchor.z - simNode.z) * k
          }
        }

        // numDimensions = 3 activates z-axis forces.
        const simulation = d3
          .forceSimulation(simNodes, 3)
          .force('charge', d3.forceManyBody().strength(-145))
          .force(
            'link', d3.forceLink(simLinks).id((d: SimNode) => d.id).distance(linkDistance).strength(0.38),
          )
          .force('cluster', clusterForce)
          .force('center', d3.forceCenter(0, 0, 0))
          .stop()

        for (let i = 0; i < 300; i++) simulation.tick()

        const posMap = new Map<string, SimNode>(simNodes.map((s) => [s.id, s]))
        const result: PositionedNode[] = newNodes.map((node) => {
          const pos = posMap.get(node.id)!
          return { ...node, x: pos.x, y: pos.y, z: pos.z }
        })

        positionedNodes.value = result
        onPositionsSettled?.(result)
      } catch (err) {
        // Force layout failed — nodes are already shown in circle layout.
        console.warn('[useForceLayout] d3-force-3d unavailable, using circle layout:', err)
        onPositionsSettled?.(positionedNodes.value as PositionedNode[])
      }
    },
    { immediate: true },
  )

  return { positionedNodes }
}
