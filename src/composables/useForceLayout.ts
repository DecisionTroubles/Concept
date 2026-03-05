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

// ---------------------------------------------------------------------------
// Instant layout — shown on the first render tick so nodes are always visible
// even while the force simulation is computing in the background.
// ---------------------------------------------------------------------------
function instantLayout(nodes: Node[]): PositionedNode[] {
  const n = nodes.length
  if (n === 0) return []
  const r = Math.max(8, n * 1.2)
  return nodes.map((node, i) => ({
    ...node,
    x: node.pos_x ?? r * Math.cos((2 * Math.PI * i) / n),
    y: node.pos_y ?? 0,
    z: node.pos_z ?? r * Math.sin((2 * Math.PI * i) / n),
  }))
}

// ---------------------------------------------------------------------------
// Main composable
// ---------------------------------------------------------------------------
export function useForceLayout(
  nodes: Ref<Node[]>,
  onPositionsSettled?: (positioned: PositionedNode[]) => void,
) {
  const positionedNodes = ref<PositionedNode[]>([])

  watch(
    nodes,
    async (newNodes) => {
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
      positionedNodes.value = instantLayout(newNodes)

      // --- Run force layout in background, then update positions ---
      try {
        // Dynamic import keeps this chunk out of the critical boot path.
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const d3: any = await import('d3-force-3d')

        const count = newNodes.length

        // Golden-angle sphere distribution for diverse initial positions.
        const simNodes: SimNode[] = newNodes.map((node, i) => {
          const phi   = Math.acos(1 - (2 * (i + 0.5)) / count)
          const theta = Math.PI * (1 + Math.sqrt(5)) * i
          const r = 12
          return {
            id: node.id,
            x: node.pos_x ?? r * Math.sin(phi) * Math.cos(theta),
            y: node.pos_y ?? r * Math.cos(phi),
            z: node.pos_z ?? r * Math.sin(phi) * Math.sin(theta),
          }
        })

        const nodeIdSet = new Set(newNodes.map((n) => n.id))
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

        // numDimensions = 3 activates z-axis forces.
        const simulation = d3
          .forceSimulation(simNodes, 3)
          .force('charge', d3.forceManyBody().strength(-160))
          .force(
            'link',
            d3.forceLink(simLinks).id((d: SimNode) => d.id).distance(6).strength(0.4),
          )
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
