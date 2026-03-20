import * as THREE from 'three'
import type { PositionedNode } from '@/composables/useForceLayout'

type JsonObject = Record<string, unknown>

export interface VisualProjectionInput {
  nodes: PositionedNode[]
  activeNodeId: string | null
  hoveredNodeId: string | null
  pinnedNodeIds: string[]
  themeVars: Record<string, string>
}

const PRIORITY_LABEL_TOKENS = ['logic', 'core', 'root', 'hub', 'center', 'central']

function nodeRadius(node: PositionedNode): number {
  return Math.min(1.4, Math.max(0.55, 0.65 + (node.weight ?? 1) * 0.2))
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

function isPriorityLabelNode(node: PositionedNode, pinned: boolean): boolean {
  if (pinned) return true
  if ((node.weight ?? 1) >= 1.35) return true
  if (node.connections.length >= 4) return true
  const title = node.title.toLowerCase()
  return PRIORITY_LABEL_TOKENS.some(token => title.includes(token))
}

export function projectVisualState(input: VisualProjectionInput) {
  const typeColors: Record<string, string> = {
    grammar: input.themeVars['--app-node-grammar'] ?? '#6aa8ff',
    kanji: input.themeVars['--app-node-kanji'] ?? '#ffb347',
    vocab: input.themeVars['--app-node-vocab'] ?? '#39e58f',
    particle: input.themeVars['--app-node-particle'] ?? '#ff7096',
    concept: input.themeVars['--app-node-concept'] ?? '#9f83ff',
    root: input.themeVars['--app-node-root'] ?? '#8fd8ff',
  }

  const emissiveByType: Record<string, string> = {
    grammar: new THREE.Color(typeColors.grammar).multiplyScalar(0.42).getStyle(),
    kanji: new THREE.Color(typeColors.kanji).multiplyScalar(0.38).getStyle(),
    vocab: new THREE.Color(typeColors.vocab).multiplyScalar(0.38).getStyle(),
    particle: new THREE.Color(typeColors.particle).multiplyScalar(0.38).getStyle(),
    concept: new THREE.Color(typeColors.concept).multiplyScalar(0.38).getStyle(),
    root: new THREE.Color(typeColors.root).multiplyScalar(0.36).getStyle(),
  }

  const nodeMap = new Map(input.nodes.map(node => [node.id, node]))
  const activeNode = input.activeNodeId ? (nodeMap.get(input.activeNodeId) ?? null) : null
  const neighborIds = new Set(
    activeNode
      ? activeNode.connections.map(conn => conn.target_id).filter(id => nodeMap.has(id))
      : []
  )
  const pinnedSet = new Set(input.pinnedNodeIds)
  const hoveredGroups = new Set(
    groupsFromTags((input.hoveredNodeId ? nodeMap.get(input.hoveredNodeId) : activeNode)?.tags ?? [])
  )

  return input.nodes.map(node => {
    const pinned = pinnedSet.has(node.id)
    const selected = input.activeNodeId === node.id
    const hovered = input.hoveredNodeId === node.id
    const neighbor = neighborIds.has(node.id)
    const groupIds = groupsFromTags(node.tags)
    const inFocusedGroup = hoveredGroups.size === 0 || groupIds.some(groupId => hoveredGroups.has(groupId))
    const baseColor = typeColors[node.node_type] ?? '#6aa8ff'
    const baseEmissive = emissiveByType[node.node_type] ?? '#2b5ec9'

    let color = baseColor
    let emissive = baseEmissive
    let emissiveIntensity = 1.05
    if (selected && pinned) {
      color = '#ffcf66'
      emissive = '#7a4a00'
      emissiveIntensity = 2.15
    } else if (selected) {
      color = '#ffffff'
      emissive = '#7f8fff'
      emissiveIntensity = 2.15
    } else if (pinned) {
      color = '#ff9f1a'
      emissive = '#6a3f00'
      emissiveIntensity = 1
    } else if (neighbor) {
      color = new THREE.Color(baseColor).lerp(new THREE.Color('#8fd4ff'), 0.55).getStyle()
      emissive = '#1a4aee'
      emissiveIntensity = 1.1
    } else if (node.learned) {
      color = '#3dd68c'
      emissive = '#1a6644'
      emissiveIntensity = 0.8
    } else if (!inFocusedGroup) {
      color = new THREE.Color(baseColor).multiplyScalar(0.8).getStyle()
      emissiveIntensity = 0.68
    }

    return {
      node,
      parentNodeId: node.parent_node_id ?? null,
      radius: nodeRadius(node),
      scale: hovered ? 1.25 : hoveredGroups.size > 0 && inFocusedGroup ? 1.09 : 1,
      color,
      emissive,
      emissiveIntensity,
      neighbor,
      hovered,
      selected,
      pinned,
      progressStatus:
        node.progress_status === 'learning' || node.progress_status === 'review' || node.progress_status === 'mastered'
          ? node.progress_status
          : 'new',
      labelPriority: isPriorityLabelNode(node, pinned) ? ('high' as const) : ('normal' as const),
    }
  })
}
