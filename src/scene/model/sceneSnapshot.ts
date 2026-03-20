import type { PositionedNode } from '@/composables/useForceLayout'

export interface SceneSnapshotEdge {
  id: string
  sourceId: string
  targetId: string
  points: [number, number, number][]
  color: string
  width: number
  opacity: number
  dashed: boolean
  dashSize: number
  gapSize: number
  dashScale: number
}

export interface SceneSnapshotNode extends PositionedNode {
  parentNodeId: string | null
  radius: number
  scale: number
  color: string
  emissive: string
  emissiveIntensity: number
  neighbor: boolean
  hovered: boolean
  selected: boolean
  pinned: boolean
  progressStatus: 'new' | 'learning' | 'review' | 'mastered'
  labelPriority: 'high' | 'normal'
}

export interface SceneSnapshot {
  mode: 'world' | 'solar'
  nodes: SceneSnapshotNode[]
  edges: SceneSnapshotEdge[]
  activeNodeId: string | null
  focusRootNodeId: string | null
  hoveredNodeId: string | null
}
