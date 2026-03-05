<script setup lang="ts">
import { Html, Line2, OrbitControls } from '@tresjs/cientos'
import { useTres } from '@tresjs/core'
import { useRafFn } from '@vueuse/core'
import * as THREE from 'three'
import { computed, onMounted, shallowRef, watch, ref } from 'vue'
import { useForceLayout, type PositionedNode } from '@/composables/useForceLayout'
import { COMPASS_RING_R, type CompassDot } from '@/composables/useEditorMode'
import { useTheme } from '@/composables/useTheme'

const graphStore = useGraphStore()
const controlsRef = shallowRef()
const coreLightRef = shallowRef<THREE.PointLight | null>(null)
const themeState = useTheme()

// TresJS context вЂ” used for fog setup.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const tres = useTres() as any

// в”Ђв”Ђ Editor mode в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const editorMode = useEditorMode()

// в”Ђв”Ђ Settings (configurable keybindings) в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const settings = useSettings()

// в”Ђв”Ђ Fly key tracking (only active in fly mode) в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const activeKeys = new Set<string>()

// Sync mode when node deselected from outside (X button, layer switch, etc.)
watch(
  () => graphStore.selectedNodeId,
  id => editorMode.onNodeSelected(id)
)

// Clear stuck fly keys when leaving fly mode
watch(
  () => editorMode.mode.value,
  m => {
    if (m !== 'fly') activeKeys.clear()
  }
)

// IDs of nodes directly connected to the selected node
const neighborIds = computed<Set<string>>(() => {
  const sel = graphStore.selectedNode
  if (!sel) return new Set()
  return new Set(sel.connections.map(c => c.target_id))
})

// Focus camera when search requests focus (even for the same node re-selected)
watch(
  () => graphStore.focusVersion,
  () => {
    const id = graphStore.selectedNodeId
    if (id) {
      const t = positionedNodes.value.find(n => n.id === id)
      if (t) focusTarget.value = new THREE.Vector3(t.x, t.y, t.z)
    }
  }
)

onMounted(() => {
  window.addEventListener('pointerdown', () => {
    // If the user starts interacting manually, stop auto-focus lerp immediately.
    focusTarget.value = null
  })

  window.addEventListener('keydown', e => {
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    const key = e.key.toLowerCase()

    if (!isInput && key === settings.keys.pinnedBuffer) {
      e.preventDefault()
      graphStore.toggleBuffer('pinned')
      return
    }
    if (!isInput && key === settings.keys.mapBuffer) {
      e.preventDefault()
      graphStore.toggleBuffer('map')
      return
    }
    if (!isInput && e.key === 'Escape' && graphStore.activeBuffer !== 'none') {
      e.preventDefault()
      graphStore.closeBuffer()
      return
    }

    if (graphStore.activeBuffer !== 'none') return

    // Fly key tracking
    if (editorMode.mode.value === 'fly') {
      const k = key
      const flyMoveKeys = [
        settings.keys.flyForward,
        settings.keys.flyBack,
        settings.keys.flyLeft,
        settings.keys.flyRight,
        settings.keys.flyUp,
        settings.keys.flyDown,
      ]
      if (flyMoveKeys.includes(k)) {
        activeKeys.add(k)
        e.preventDefault()
      }
    }

    if (e.key === 'Escape') {
      e.preventDefault()
      editorMode.escapeFromCurrentMode()
      return
    }
    if (!isInput && key === settings.keys.flyMode) {
      editorMode.enterFly()
      return
    }
    if (!isInput && key === settings.keys.graphMode && graphStore.selectedNodeId) {
      editorMode.enterGraph()
      return
    }
    if (!isInput && graphStore.selectedNodeId && key === settings.keys.openNode) {
      e.preventDefault()
      graphStore.toggleCenteredNodePanel()
      return
    }
    if (!isInput && graphStore.selectedNodeId && key === settings.keys.pinNode) {
      e.preventDefault()
      graphStore.togglePinNode(graphStore.selectedNodeId)
      return
    }

    if (editorMode.mode.value !== 'fly' && !isInput) {
      const rawControls = controlsRef.value
      const controls = rawControls?.instance ?? rawControls
      const cam = controls?.object as THREE.PerspectiveCamera | undefined

      if (controls?.target && cam) {
        const orbitStep = 0.12
        const tiltStep = 0.09
        const zoomInFactor = 0.9
        const zoomOutFactor = 1.12
        let changed = false

        _orbitOffset.copy(cam.position).sub(controls.target)
        _orbitSpherical.setFromVector3(_orbitOffset)

        if (key === settings.keys.graphOrbitLeft) {
          e.preventDefault()
          _orbitSpherical.theta += orbitStep
          changed = true
        } else if (key === settings.keys.graphOrbitRight) {
          e.preventDefault()
          _orbitSpherical.theta -= orbitStep
          changed = true
        } else if (key === settings.keys.graphTiltUp) {
          e.preventDefault()
          _orbitSpherical.phi = Math.max(0.2, _orbitSpherical.phi - tiltStep)
          changed = true
        } else if (key === settings.keys.graphTiltDown) {
          e.preventDefault()
          _orbitSpherical.phi = Math.min(Math.PI - 0.2, _orbitSpherical.phi + tiltStep)
          changed = true
        } else if (key === settings.keys.graphZoomIn) {
          e.preventDefault()
          _orbitSpherical.radius = Math.max(3, _orbitSpherical.radius * zoomInFactor)
          changed = true
        } else if (key === settings.keys.graphZoomOut) {
          e.preventDefault()
          _orbitSpherical.radius = Math.min(140, _orbitSpherical.radius * zoomOutFactor)
          changed = true
        }

        if (changed) {
          _orbitOffset.setFromSpherical(_orbitSpherical)
          cam.position.copy(controls.target).add(_orbitOffset)
          controls.update()
          focusTarget.value = null
          return
        }
      }
    }

    if (editorMode.mode.value === 'graph' && !isInput) {
      if (e.key === 'Tab') {
        e.preventDefault()
        const id = e.shiftKey ? editorMode.tabPrev() : editorMode.tabNext()
        if (id) {
          graphStore.selectNode(id)
          const t = positionedNodes.value.find(n => n.id === id)
          if (t) focusTarget.value = new THREE.Vector3(t.x, t.y, t.z)
        }
        return
      }
      const num = parseInt(e.key)
      if (num >= 1 && num <= 9) {
        e.preventDefault()
        const id = editorMode.jumpToNeighbor(num)
        if (id) {
          graphStore.selectNode(id)
          const t = positionedNodes.value.find(n => n.id === id)
          if (t) focusTarget.value = new THREE.Vector3(t.x, t.y, t.z)
        }
      }
    }

    // jump back (works in any non-fly mode)
    if (!isInput && key === settings.keys.jumpBack && editorMode.mode.value !== 'fly') {
      e.preventDefault()
      const id = editorMode.jumpBack()
      if (id) {
        graphStore.selectNode(id)
        const t = positionedNodes.value.find(n => n.id === id)
        if (t) focusTarget.value = new THREE.Vector3(t.x, t.y, t.z)
      } else {
        editorMode.escapeFromCurrentMode()
      }
      return
    }
  })

  window.addEventListener('keyup', e => {
    activeKeys.delete(e.key.toLowerCase())
  })

  // Scene fog вЂ” gives the graph depth and a sense of infinite space.
  try {
    const scene: THREE.Scene | undefined = tres?.scene?.value ?? tres?.scene
    if (scene instanceof THREE.Scene) {
      scene.fog = new THREE.FogExp2(new THREE.Color('#080b14'), 0.014)
    }
  } catch {
    /* skip if context not yet ready */
  }
})

// в”Ђв”Ђ Camera focus animation в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const focusTarget = shallowRef<THREE.Vector3 | null>(null)

const _fwd = new THREE.Vector3()
const _right = new THREE.Vector3()
const _move = new THREE.Vector3()
const _up = new THREE.Vector3(0, 1, 0)
const _camLerpTarget = new THREE.Vector3()
const _orbitLerpTarget = new THREE.Vector3()
const _orbitOffset = new THREE.Vector3()
const _orbitSpherical = new THREE.Spherical()

// Pulse state for the core light
let pulseT = 0

// Camera position snapshot for distance-faded labels (sampled, not every frame)
const cameraPos = new THREE.Vector3()
const labelTick = ref(0)
let labelSampleMs = 0

const _ndcVec = new THREE.Vector3()

useRafFn(({ delta }) => {
  const raw = controlsRef.value
  const controls = raw?.instance ?? raw
  if (!controls?.object) return

  const cam = controls.object as THREE.PerspectiveCamera
  const speed = 14 * (delta / 1000)

  // Fly mode movement вЂ” only when in fly mode and keys are held
  if (editorMode.mode.value === 'fly' && activeKeys.size > 0) {
    cam.getWorldDirection(_fwd)
    _fwd.y = 0
    _fwd.normalize()
    _right.crossVectors(_fwd, _up).normalize()
    _move.set(0, 0, 0)

    if (activeKeys.has(settings.keys.flyForward)) _move.addScaledVector(_fwd, speed)
    if (activeKeys.has(settings.keys.flyBack)) _move.addScaledVector(_fwd, -speed)
    if (activeKeys.has(settings.keys.flyLeft)) _move.addScaledVector(_right, -speed)
    if (activeKeys.has(settings.keys.flyRight)) _move.addScaledVector(_right, speed)
    if (activeKeys.has(settings.keys.flyDown)) _move.y -= speed
    if (activeKeys.has(settings.keys.flyUp)) _move.y += speed

    cam.position.add(_move)
    controls.target.add(_move)
    controls.update()
  }

  // Camera lerp toward selected node
  if (focusTarget.value) {
    _camLerpTarget.set(focusTarget.value.x, focusTarget.value.y + 3, focusTarget.value.z + 11)
    _orbitLerpTarget.copy(focusTarget.value)

    cam.position.lerp(_camLerpTarget, 0.06)
    controls.target.lerp(_orbitLerpTarget, 0.06)
    controls.update()

    if (cam.position.distanceTo(_camLerpTarget) < 0.2) focusTarget.value = null
  }

  // Animate central core light
  if (coreLightRef.value) {
    pulseT += delta / 1000
    coreLightRef.value.intensity = 55 + 22 * Math.sin(pulseT * 0.4)
  }

  // Sample camera position at a lower rate to avoid forcing heavy full-scene reactivity.
  labelSampleMs += delta
  if (labelSampleMs >= 120) {
    cameraPos.copy(cam.position)
    labelTick.value++
    labelSampleMs = 0
  }

  // Compass projection (graph mode only)
  if (editorMode.mode.value === 'graph' && graphStore.selectedNodeId) {
    const sel = positionedNodes.value.find(n => n.id === graphStore.selectedNodeId)
    if (sel) {
      _ndcVec.set(sel.x, sel.y, sel.z).project(cam)
      const sx = ((_ndcVec.x + 1) / 2) * window.innerWidth
      const sy = ((-_ndcVec.y + 1) / 2) * window.innerHeight
      const center = { x: sx, y: sy }
      const nodeMap = new Map(positionedNodes.value.map(n => [n.id, n]))
      // Deduplicate by target_id (same neighbor can appear via both an outgoing
      // and an incoming edge after the bidirectional query), then filter to
      // nodes present in this layer, so indices are always sequential 1, 2, 3вЂ¦
      const seen = new Set<string>()
      const validConns = sel.connections
        .filter(conn => {
          if (!nodeMap.has(conn.target_id) || seen.has(conn.target_id)) return false
          if (
            graphStore.connectionLayers.length > 0 &&
            (
              graphStore.activeConnectionLayerIds.length === 0 ||
              conn.connection_layer_ids.length === 0 ||
              !conn.connection_layer_ids.some(id => graphStore.activeConnectionLayerIds.includes(id))
            )
          ) {
            return false
          }
          seen.add(conn.target_id)
          return true
        })
      editorMode.setNeighborOrder(validConns.map(conn => conn.target_id))

      const provisional = validConns.slice(0, 9).map((conn, i) => {
        const nb = nodeMap.get(conn.target_id)!
        _ndcVec.set(nb.x, nb.y, nb.z).project(cam)
        const nx = ((_ndcVec.x + 1) / 2) * window.innerWidth
        const ny = ((-_ndcVec.y + 1) / 2) * window.innerHeight
        const angle = Math.atan2(ny - sy, nx - sx)
        return { conn, i, angle, title: nb.title }
      })
      provisional.sort((a, b) => a.angle - b.angle)

      const minAngularGap = 0.23
      const dots: CompassDot[] = provisional.map((item, idx) => {
        let ring = COMPASS_RING_R
        if (idx > 0) {
          const prev = provisional[idx - 1]
          const gap = Math.abs(item.angle - prev.angle)
          if (gap < minAngularGap) {
            ring += 16 + (minAngularGap - gap) * 55
          }
        }
        return {
          id: item.conn.target_id,
          title: item.title,
          screenX: sx + Math.cos(item.angle) * ring,
          screenY: sy + Math.sin(item.angle) * ring,
          edgeType: item.conn.edge_type,
          index: item.i + 1,
        }
      })
      editorMode.setCompassState(dots, center)
    }
  } else {
    editorMode.setNeighborOrder([])
    editorMode.setCompassState([], null)
  }
})

// в”Ђв”Ђ Hover state в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const hoveredNodeId = ref<string | null>(null)

type JsonObject = Record<string, unknown>
type GroupStyleConfig = { color: string; emissive: string; emissiveIntensity: number }
type GroupLayoutConfig = { cohesion: number; intraSpacing: number }
type GroupRelationStyleConfig = {
  color: string
  width: number
  opacity: number
  dashSize: number
  gapSize: number
}

function parseJsonObject(raw: string | null | undefined): JsonObject {
  if (!raw) return {}
  try {
    const parsed = JSON.parse(raw)
    return parsed && typeof parsed === 'object' ? (parsed as JsonObject) : {}
  } catch {
    return {}
  }
}

function strOr(obj: JsonObject, key: string, fallback: string): string {
  const value = obj[key]
  return typeof value === 'string' ? value : fallback
}

function numOr(obj: JsonObject, key: string, fallback: number): number {
  const value = obj[key]
  return typeof value === 'number' ? value : fallback
}

function boolOr(obj: JsonObject, key: string, fallback: boolean): boolean {
  const value = obj[key]
  return typeof value === 'boolean' ? value : fallback
}

const worldRoot = computed(() => parseJsonObject(graphStore.worldConfig?.config_json))

const worldMetadata = computed<JsonObject>(() => {
  const metadata = worldRoot.value.metadata
  return metadata && typeof metadata === 'object' ? (metadata as JsonObject) : {}
})

const contextGroups = computed(() => {
  const raw = worldMetadata.value.context_groups
  const fallbackPalette = [
    '#ff4d6d',
    '#22c55e',
    '#38bdf8',
    '#f59e0b',
    '#a78bfa',
    '#14b8a6',
    '#fb7185',
    '#84cc16',
  ]
  const out = new Map<string, { style: GroupStyleConfig; layout: GroupLayoutConfig }>()

  if (!raw || typeof raw !== 'object') return out
  const entries = Object.entries(raw as JsonObject)
  for (let i = 0; i < entries.length; i++) {
    const [groupId, def] = entries[i]
    if (!def || typeof def !== 'object') continue
    const obj = def as JsonObject
    const styleRaw = obj.style && typeof obj.style === 'object' ? (obj.style as JsonObject) : {}
    const layoutRaw = obj.layout && typeof obj.layout === 'object' ? (obj.layout as JsonObject) : {}
    const color = strOr(styleRaw, 'color', fallbackPalette[i % fallbackPalette.length])
    const emissive = strOr(
      styleRaw,
      'emissive',
      new THREE.Color(color).clone().multiplyScalar(0.36).getStyle(),
    )
    out.set(groupId, {
      style: {
        color,
        emissive,
        emissiveIntensity: numOr(styleRaw, 'emissive_intensity', 1.06),
      },
      layout: {
        cohesion: numOr(layoutRaw, 'cohesion', 1),
        intraSpacing: numOr(layoutRaw, 'intra_spacing', 6.2),
      },
    })
  }
  return out
})

const groupRelationStyles = computed(() => {
  const raw = worldMetadata.value.group_relations
  const empty: { intra: GroupRelationStyleConfig | null; bridge: GroupRelationStyleConfig | null } = {
    intra: null,
    bridge: null,
  }
  if (!raw || typeof raw !== 'object') return empty

  const parseStyle = (input: unknown, defaults: GroupRelationStyleConfig): GroupRelationStyleConfig | null => {
    if (!input || typeof input !== 'object') return null
    const src = input as JsonObject
    return {
      color: strOr(src, 'color', defaults.color),
      width: numOr(src, 'width', defaults.width),
      opacity: numOr(src, 'opacity', defaults.opacity),
      dashSize: numOr(src, 'dash_size', defaults.dashSize),
      gapSize: numOr(src, 'gap_size', defaults.gapSize),
    }
  }

  const obj = raw as JsonObject
  return {
    intra: parseStyle(obj.intra, {
      color: '#7482a5',
      width: 1.7,
      opacity: 0.95,
      dashSize: 0,
      gapSize: 0,
    }),
    bridge: parseStyle(obj.bridge, {
      color: '#ffb347',
      width: 2.15,
      opacity: 0.98,
      dashSize: 0.24,
      gapSize: 0.14,
    }),
  }
})

function groupsFromTags(tags: string[]): string[] {
  const out: string[] = []
  for (const tag of tags) {
    const lower = tag.toLowerCase()
    if (lower.startsWith('group:')) out.push(tag.slice(6))
    else if (lower.startsWith('cluster:')) out.push(tag.slice(8))
  }
  return Array.from(new Set(out))
}

const nodeGroupsById = computed(() => {
  const map = new Map<string, string[]>()
  for (const node of graphStore.nodes) {
    map.set(node.id, groupsFromTags(node.tags))
  }
  return map
})

const layoutClusterOptions = computed(() => {
  const membershipsByNodeId: Record<string, string[]> = {}
  const groupCohesionById: Record<string, number> = {}
  const groupIntraSpacingById: Record<string, number> = {}

  for (const node of graphStore.nodes) {
    const ids = nodeGroupsById.value.get(node.id) ?? []
    if (ids.length) membershipsByNodeId[node.id] = ids
  }

  for (const [groupId, def] of contextGroups.value.entries()) {
    groupCohesionById[groupId] = def.layout.cohesion
    groupIntraSpacingById[groupId] = def.layout.intraSpacing
  }

  const hasGroups = Object.keys(membershipsByNodeId).length > 0
  if (!hasGroups) return undefined
  return {
    cluster: {
      membershipsByNodeId,
      groupCohesionById,
      groupIntraSpacingById,
      interGroupSpacing: numOr(worldMetadata.value, 'inter_group_spacing', 24),
    },
  }
})

const worldVisual = computed(() => {
  const metadata = worldMetadata.value
  return metadata.visual_defaults && typeof metadata.visual_defaults === 'object'
    ? (metadata.visual_defaults as JsonObject)
    : {}
})

const nodeTypeStyles = computed(() => {
  const metadata = worldMetadata.value
  return metadata.node_type_styles && typeof metadata.node_type_styles === 'object'
    ? (metadata.node_type_styles as JsonObject)
    : {}
})

const activeNodeLayerStyle = computed<JsonObject>(() => {
  const layer = graphStore.layers.find(l => l.id === graphStore.activeLayerId)
  if (!layer) return {}
  const metadata = parseJsonObject(layer.metadata)
  return metadata.node_style && typeof metadata.node_style === 'object' ? (metadata.node_style as JsonObject) : {}
})

const relationStyleById = computed(() => {
  const map = new Map<string, JsonObject>()
  for (const relation of graphStore.relationKinds) {
    const metadata = parseJsonObject(relation.metadata)
    const style = metadata.style && typeof metadata.style === 'object' ? (metadata.style as JsonObject) : {}
    map.set(relation.id, style)
  }
  return map
})

const connectionLayerById = computed(() => {
  const map = new Map<string, { order: number; style: JsonObject }>()
  for (const layer of graphStore.connectionLayers) {
    const metadata = parseJsonObject(layer.metadata)
    const style =
      metadata.edge_style && typeof metadata.edge_style === 'object' ? (metadata.edge_style as JsonObject) : {}
    map.set(layer.id, { order: layer.display_order, style })
  }
  return map
})

// в”Ђв”Ђ Force layout (cluster-aware when groups are present) в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const { positionedNodes } = useForceLayout(
  computed(() => graphStore.nodes),
  computed(() => layoutClusterOptions.value),
  settled => {
    for (const node of settled) {
      graphStore.updateNodePosition(node.id, node.x, node.y, node.z)
    }
  }
)

// в”Ђв”Ђ Node helpers в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
function nodeRadius(node: PositionedNode): number {
  return Math.min(1.4, Math.max(0.55, 0.65 + (node.weight ?? 1) * 0.2))
}

const TYPE_COLORS = computed<Record<string, string>>(() => {
  const vars = themeState.activeTheme.value?.vars ?? {}
  return {
    grammar: vars['--app-node-grammar'] ?? '#6aa8ff',
    kanji: vars['--app-node-kanji'] ?? '#ffb347',
    vocab: vars['--app-node-vocab'] ?? '#39e58f',
    particle: vars['--app-node-particle'] ?? '#ff7096',
    concept: vars['--app-node-concept'] ?? '#9f83ff',
    root: vars['--app-node-root'] ?? '#8fd8ff',
  }
})

const TYPE_EMISSIVE = computed<Record<string, string>>(() => {
  const colors = TYPE_COLORS.value
  return {
    grammar: new THREE.Color(colors.grammar).multiplyScalar(0.42).getStyle(),
    kanji: new THREE.Color(colors.kanji).multiplyScalar(0.38).getStyle(),
    vocab: new THREE.Color(colors.vocab).multiplyScalar(0.38).getStyle(),
    particle: new THREE.Color(colors.particle).multiplyScalar(0.38).getStyle(),
    concept: new THREE.Color(colors.concept).multiplyScalar(0.38).getStyle(),
    root: new THREE.Color(colors.root).multiplyScalar(0.36).getStyle(),
  }
})

function nodeGroupIds(node: PositionedNode): string[] {
  return nodeGroupsById.value.get(node.id) ?? []
}

function nodePrimaryGroup(node: PositionedNode): string | null {
  const groups = nodeGroupIds(node)
  if (groups.length === 0) return null
  for (const g of groups) {
    if (contextGroups.value.has(g)) return g
  }
  return groups[0] ?? null
}

function nodeSharesGroup(a: PositionedNode, b: PositionedNode): boolean {
  const ag = nodeGroupIds(a)
  const bg = nodeGroupIds(b)
  if (ag.length === 0 || bg.length === 0) return false
  const set = new Set(ag)
  return bg.some(g => set.has(g))
}

function nodeBaseByGroup(node: PositionedNode): GroupStyleConfig | null {
  const primary = nodePrimaryGroup(node)
  if (!primary) return null
  return contextGroups.value.get(primary)?.style ?? null
}

const focusedGroupSet = computed<Set<string>>(() => {
  const baseId = hoveredNodeId.value ?? graphStore.selectedNodeId
  if (!baseId) return new Set()
  const groups = nodeGroupsById.value.get(baseId) ?? []
  return new Set(groups)
})

function resolvedNodeBase(node: PositionedNode): { color: string; emissive: string; emissiveIntensity: number } {
  const defaultNode = worldVisual.value.node && typeof worldVisual.value.node === 'object' ? (worldVisual.value.node as JsonObject) : {}
  const typeNodeRaw = nodeTypeStyles.value[node.node_type]
  const typeNode = typeNodeRaw && typeof typeNodeRaw === 'object' ? (typeNodeRaw as JsonObject) : {}
  const layerNode = activeNodeLayerStyle.value
  const groupBase = nodeBaseByGroup(node)

  const themeColor = TYPE_COLORS.value[node.node_type] ?? '#6aa8ff'
  const themeEmissive = TYPE_EMISSIVE.value[node.node_type] ?? '#2b5ec9'

  // Group style is first-class and should win for grouped nodes.
  // Layer style applies only when node has no group style.
  const color = groupBase?.color ?? strOr(layerNode, 'color', themeColor)
  const emissive = groupBase?.emissive ?? strOr(layerNode, 'emissive', themeEmissive)
  const emissiveIntensity = numOr(
    layerNode,
    'emissive_intensity',
    groupBase?.emissiveIntensity ?? numOr(typeNode, 'emissive_intensity', numOr(defaultNode, 'emissive_intensity', 1.05))
  )

  return { color, emissive, emissiveIntensity }
}

function nodeColor(node: PositionedNode): string {
  const base = resolvedNodeBase(node)
  if (graphStore.selectedNodeId === node.id && graphStore.isNodePinned(node.id)) return '#ffcf66'
  if (graphStore.selectedNodeId === node.id) return '#ffffff'
  if (graphStore.isNodePinned(node.id)) return '#ff9f1a'
  if (neighborIds.value.has(node.id)) {
    return new THREE.Color(base.color).lerp(new THREE.Color('#8fd4ff'), 0.55).getStyle()
  }
  if (node.learned) return '#3dd68c'
  if (focusedGroupSet.value.size > 0) {
    const inFocusGroup = nodeGroupIds(node).some(g => focusedGroupSet.value.has(g))
    if (!inFocusGroup) return new THREE.Color(base.color).multiplyScalar(0.8).getStyle()
    return new THREE.Color(base.color).lerp(new THREE.Color('#ffffff'), 0.12).getStyle()
  }
  return base.color
}

function nodeEmissive(node: PositionedNode): string {
  const base = resolvedNodeBase(node)
  if (graphStore.selectedNodeId === node.id && graphStore.isNodePinned(node.id)) return '#7a4a00'
  if (graphStore.selectedNodeId === node.id) return '#7f8fff'
  if (graphStore.isNodePinned(node.id)) return '#6a3f00'
  if (neighborIds.value.has(node.id)) return '#1a4aee'
  if (node.learned) return '#1a6644'
  return base.emissive
}

function nodeEmissiveIntensity(node: PositionedNode): number {
  const base = resolvedNodeBase(node)
  if (graphStore.selectedNodeId === node.id) return 2.15
  if (graphStore.isNodePinned(node.id)) return 1.0
  if (neighborIds.value.has(node.id)) return 1.1
  if (node.learned) return 0.8
  if (focusedGroupSet.value.size > 0) {
    const inFocusGroup = nodeGroupIds(node).some(g => focusedGroupSet.value.has(g))
    return inFocusGroup ? base.emissiveIntensity : base.emissiveIntensity * 0.65
  }
  return base.emissiveIntensity
}

function nodeScale(node: PositionedNode): number {
  if (focusedGroupSet.value.size > 0 && nodeGroupIds(node).some(g => focusedGroupSet.value.has(g))) {
    return hoveredNodeId.value === node.id ? 1.3 : 1.09
  }
  return hoveredNodeId.value === node.id ? 1.25 : 1.0
}

// в”Ђв”Ђ Edge helpers в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
function edgeColor(edgeType: string): string {
  switch (edgeType) {
    case 'Prerequisite':
      return '#5b8fff'
    case 'Semantic':
      return '#5a648c'
    case 'UserDefined':
      return '#f59e0b'
    case 'Context':
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

function isBridgeEdge(source: PositionedNode, target: PositionedNode): boolean {
  const sourceGroups = nodeGroupIds(source)
  const targetGroups = nodeGroupIds(target)
  if (sourceGroups.length === 0 || targetGroups.length === 0) return false
  return !nodeSharesGroup(source, target)
}

function resolvedEdgeStyle(
  conn: { edge_type: string; relation_id: string | null; connection_layer_ids: string[] },
  activeConnectionLayerSet: Set<string>,
  isBridge: boolean
): {
  color: string
  width: number
  opacity: number
  dashed: boolean
  dashSize: number
  gapSize: number
  animatedFlow: boolean
  flowSpeed: number
} {
  const defaultEdge =
    worldVisual.value.edge && typeof worldVisual.value.edge === 'object' ? (worldVisual.value.edge as JsonObject) : {}
  const relationStyle = conn.relation_id ? (relationStyleById.value.get(conn.relation_id) ?? {}) : {}

  const candidates = conn.connection_layer_ids
    .filter(id => activeConnectionLayerSet.has(id))
    .map(id => connectionLayerById.value.get(id))
    .filter((v): v is { order: number; style: JsonObject } => Boolean(v))
    .sort((a, b) => b.order - a.order)
  const topLayerStyle = candidates[0]?.style ?? {}
  const groupStyle = isBridge ? groupRelationStyles.value.bridge : groupRelationStyles.value.intra
  const groupStyleObj = (groupStyle ?? {}) as JsonObject

  return {
    color: strOr(
      topLayerStyle,
      'color',
      strOr(groupStyleObj, 'color', strOr(relationStyle, 'color', strOr(defaultEdge, 'color', edgeColor(conn.edge_type))))
    ),
    width: numOr(
      topLayerStyle,
      'width',
      numOr(groupStyleObj, 'width', numOr(relationStyle, 'width', numOr(defaultEdge, 'width', edgeLineWidth(conn.edge_type))))
    ),
    opacity: numOr(topLayerStyle, 'opacity', numOr(groupStyleObj, 'opacity', numOr(relationStyle, 'opacity', numOr(defaultEdge, 'opacity', 0.9)))),
    dashed:
      numOr(topLayerStyle, 'dash_size', numOr(groupStyleObj, 'dash_size', numOr(relationStyle, 'dash_size', numOr(defaultEdge, 'dash_size', 0)))) > 0,
    dashSize: numOr(topLayerStyle, 'dash_size', numOr(groupStyleObj, 'dash_size', numOr(relationStyle, 'dash_size', numOr(defaultEdge, 'dash_size', 0)))),
    gapSize: numOr(topLayerStyle, 'gap_size', numOr(groupStyleObj, 'gap_size', numOr(relationStyle, 'gap_size', numOr(defaultEdge, 'gap_size', 0)))),
    animatedFlow: boolOr(
      topLayerStyle,
      'animated_flow',
      boolOr(relationStyle, 'animated_flow', boolOr(defaultEdge, 'animated_flow', false))
    ),
    flowSpeed: numOr(
      topLayerStyle,
      'flow_speed',
      numOr(relationStyle, 'flow_speed', numOr(defaultEdge, 'flow_speed', 1))
    ),
  }
}

// в”Ђв”Ђ Edge list for cientos Line2 component в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const edges = computed(() => {
  const nodeMap = new Map<string, PositionedNode>(positionedNodes.value.map(n => [n.id, n]))
  const activeConnectionLayerSet = new Set(graphStore.activeConnectionLayerIds)
  const hasConnectionLayers = graphStore.connectionLayers.length > 0
  const selectedCount = activeConnectionLayerSet.size
  const result: {
    id: string
    points: [number, number, number][]
    color: string
    width: number
    opacity: number
    dashed: boolean
    dashSize: number
    gapSize: number
    dashScale: number
  }[] = []
  const seenEdgeIds = new Set<string>()
  const edgeRows: {
    id: string
    source: PositionedNode
    target: PositionedNode
    color: string
    width: number
    opacity: number
    dashed: boolean
    dashSize: number
    gapSize: number
    dashScale: number
    pairKey: string
  }[] = []

  for (const node of positionedNodes.value) {
    for (const conn of node.connections) {
      if (seenEdgeIds.has(conn.id)) continue
      seenEdgeIds.add(conn.id)

      if (
        hasConnectionLayers &&
        (
          selectedCount === 0 ||
          conn.connection_layer_ids.length === 0 ||
          !conn.connection_layer_ids.some(id => activeConnectionLayerSet.has(id))
        )
      ) {
        continue
      }

      const target = nodeMap.get(conn.target_id)
      if (!target) continue
      const style = resolvedEdgeStyle(conn, activeConnectionLayerSet, isBridgeEdge(node, target))

      const a = node.id < conn.target_id ? node.id : conn.target_id
      const b = node.id < conn.target_id ? conn.target_id : node.id
      edgeRows.push({
        id: conn.id,
        source: node,
        target,
        color: style.color,
        width: style.width,
        opacity: Math.max(0.12, Math.min(1, style.opacity)),
        dashed: style.dashed,
        dashSize: style.dashSize > 0 ? style.dashSize : 0.22,
        gapSize: style.gapSize > 0 ? style.gapSize : 0.14,
        dashScale: style.flowSpeed > 0 ? style.flowSpeed : 1,
        pairKey: `${a}::${b}`,
      })
    }
  }

  const bundles = new Map<string, typeof edgeRows>()
  for (const row of edgeRows) {
    const bucket = bundles.get(row.pairKey)
    if (bucket) bucket.push(row)
    else bundles.set(row.pairKey, [row])
  }

  const up = new THREE.Vector3(0, 1, 0)
  const xAxis = new THREE.Vector3(1, 0, 0)
  const dir = new THREE.Vector3()
  const normal = new THREE.Vector3()
  const mid = new THREE.Vector3()

  for (const bundle of bundles.values()) {
    const count = bundle.length
    for (let i = 0; i < count; i++) {
      const row = bundle[i]
      const lane = i - (count - 1) / 2
      const offsetStrength = 0.38
      const src = row.source
      const tgt = row.target

      if (count > 1 && lane !== 0) {
        dir.set(tgt.x - src.x, tgt.y - src.y, tgt.z - src.z)
        if (dir.lengthSq() < 1e-6) continue
        dir.normalize()
        normal.crossVectors(dir, up)
        if (normal.lengthSq() < 1e-6) normal.crossVectors(dir, xAxis)
        normal.normalize().multiplyScalar(offsetStrength * lane)
        mid.set((src.x + tgt.x) / 2, (src.y + tgt.y) / 2, (src.z + tgt.z) / 2).add(normal)

        result.push({
          id: row.id,
          points: [
            [src.x, src.y, src.z],
            [mid.x, mid.y, mid.z],
            [tgt.x, tgt.y, tgt.z],
          ],
          color: row.color,
          width: row.width,
          opacity: row.opacity,
          dashed: row.dashed,
          dashSize: row.dashSize,
          gapSize: row.gapSize,
          dashScale: row.dashScale,
        })
      } else {
        result.push({
          id: row.id,
          points: [
            [src.x, src.y, src.z],
            [tgt.x, tgt.y, tgt.z],
          ],
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
  }

  return result
})

// в”Ђв”Ђ Distance-faded label opacity в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
const PRIORITY_LABEL_TOKENS = ['logic', 'core', 'root', 'hub', 'center', 'central']

function isPriorityLabelNode(node: PositionedNode): boolean {
  if (graphStore.isNodePinned(node.id)) return true
  if ((node.weight ?? 1) >= 1.35) return true
  if (node.connections.length >= 4) return true
  const title = node.title.toLowerCase()
  return PRIORITY_LABEL_TOKENS.some(token => title.includes(token))
}

function nodeLabelOpacity(node: PositionedNode): number {
  // make opacity recompute only on sampled camera updates
  void labelTick.value
  if (hoveredNodeId.value === node.id || graphStore.selectedNodeId === node.id) return 1
  const dx = cameraPos.x - node.x
  const dy = cameraPos.y - node.y
  const dz = cameraPos.z - node.z
  const dist = Math.sqrt(dx * dx + dy * dy + dz * dz)
  if (isPriorityLabelNode(node)) {
    if (dist <= 20) return 1
    if (dist >= 78) return 0
    return 1 - (dist - 20) / 58
  }
  if (dist <= 14) return 1
  if (dist >= 42) return 0
  return 1 - (dist - 14) / 28
}

// в”Ђв”Ђ Event handlers в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
function onNodeClick(node: PositionedNode, event: { stopPropagation?: () => void }) {
  event.stopPropagation?.()
  graphStore.selectNode(node.id)
  focusTarget.value = new THREE.Vector3(node.x, node.y, node.z)
  editorMode.onNodeSelected(node.id)
}

function onNodePointerEnter(node: PositionedNode, event: { stopPropagation?: () => void }) {
  event.stopPropagation?.()
  hoveredNodeId.value = node.id
}

function onNodePointerLeave(node: PositionedNode, event: { stopPropagation?: () => void }) {
  event.stopPropagation?.()
  if (hoveredNodeId.value === node.id) hoveredNodeId.value = null
}

// Watch for layer changes вЂ” reset focus
watch(
  () => graphStore.activeLayerId,
  () => {
    focusTarget.value = null
    hoveredNodeId.value = null
  }
)
</script>

<template>
  <!-- Camera -->
  <TresPerspectiveCamera :position="[0, 8, 28]" :fov="60" />
  <OrbitControls ref="controlsRef" enable-damping :damping-factor="0.05" />

  <!-- в”Ђв”Ђ Lighting в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ -->
  <TresHemisphereLight sky-color="#2a3a66" ground-color="#08080f" :intensity="0.7" />
  <TresDirectionalLight :position="[12, 20, 8]" color="#ccd8ff" :intensity="1.8" />
  <TresPointLight
    ref="coreLightRef"
    :position="[0, 0, 0]"
    color="#4466ee"
    :intensity="55"
    :distance="42"
    :decay="1.8"
  />
  <TresPointLight :position="[0, 16, 0]" color="#5566ff" :intensity="40" :distance="50" :decay="1.5" />
  <TresPointLight :position="[-14, -7, 12]" color="#ff3366" :intensity="28" :distance="40" :decay="2" />
  <TresPointLight :position="[14, -5, -10]" color="#33aaff" :intensity="18" :distance="35" :decay="2" />

  <!-- в”Ђв”Ђ Edges в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ -->
  <Line2
    v-for="edge in edges"
    :key="edge.id"
    :points="edge.points"
    :color="edge.color"
    :line-width="edge.width"
    :opacity="edge.opacity"
    :transparent="edge.opacity < 0.999"
    :dashed="edge.dashed"
    :dash-size="edge.dashSize"
    :gap-size="edge.gapSize"
    :dash-scale="edge.dashScale"
  />

  <!-- в”Ђв”Ђ Nodes в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ -->
  <TresMesh
    v-for="node in positionedNodes"
    :key="node.id"
    :position="[node.x, node.y, node.z]"
    :scale="nodeScale(node)"
    @click="(e: any) => onNodeClick(node, e)"
    @pointer-enter="(e: any) => onNodePointerEnter(node, e)"
    @pointer-leave="(e: any) => onNodePointerLeave(node, e)"
  >
    <TresOctahedronGeometry v-if="node.node_type === 'grammar'" :args="[nodeRadius(node) * 0.85, 0]" />
    <TresBoxGeometry
      v-else-if="node.node_type === 'kanji'"
      :args="[nodeRadius(node) * 1.2, nodeRadius(node) * 1.2, nodeRadius(node) * 1.2]"
    />
    <TresIcosahedronGeometry v-else-if="node.node_type === 'concept'" :args="[nodeRadius(node) * 0.9, 0]" />
    <TresTorusGeometry
      v-else-if="node.node_type === 'particle'"
      :args="[nodeRadius(node) * 0.7, nodeRadius(node) * 0.22, 12, 24]"
    />
    <TresSphereGeometry v-else :args="[nodeRadius(node), 18, 14]" />
    <TresMeshStandardMaterial
      :color="nodeColor(node)"
      :emissive="nodeEmissive(node)"
      :emissive-intensity="nodeEmissiveIntensity(node)"
      :roughness="0.35"
      :metalness="0.4"
    />
  </TresMesh>

  <!-- Soft group halo per node (distance-readable, low clutter) -->
  <TresMesh
    v-for="node in positionedNodes"
    :key="`halo-${node.id}`"
    :position="[node.x, node.y, node.z]"
    :scale="nodeScale(node) * 1.05"
  >
    <TresSphereGeometry :args="[nodeRadius(node) * 1.8, 14, 10]" />
    <TresMeshBasicMaterial
      :color="nodeColor(node)"
      :opacity="focusedGroupSet.size > 0 && !nodeGroupIds(node).some(g => focusedGroupSet.has(g)) ? 0.08 : 0.16"
      transparent
      :depth-write="false"
    />
  </TresMesh>

  <!-- в”Ђв”Ђ Node labels (all nodes, distance-faded) в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ -->
  <Html
    v-for="node in positionedNodes"
    :key="`label-${node.id}`"
    :position="[node.x, node.y + nodeRadius(node) + 0.7, node.z]"
    center
    :sprite="true"
    :z-index-range="[40, 0]"
  >
    <div
      v-if="nodeLabelOpacity(node) > 0.01"
      class="node-label"
      :style="{ opacity: nodeLabelOpacity(node) }"
    >
      {{ node.title }}
    </div>
  </Html>

  <Html
    v-for="node in positionedNodes.filter(n => graphStore.isNodePinned(n.id))"
    :key="`pin-tag-${node.id}`"
    :position="[node.x, node.y + nodeRadius(node) + 1.3, node.z]"
    center
    :sprite="true"
    :z-index-range="[4, 0]"
    occlude
  >
    <div class="pin-tag">Pinned</div>
  </Html>
</template>

<style scoped>
.node-label {
  color: #e8eaf0;
  font-family: 'Inter', 'Segoe UI Variable', 'Noto Sans JP', 'Segoe UI', sans-serif;
  font-size: clamp(12px, 0.72vw, 15px);
  font-weight: 600;
  letter-spacing: 0.03em;
  white-space: nowrap;
  pointer-events: none;
  text-shadow:
    0 0 6px rgba(0, 0, 0, 0.95),
    0 0 14px rgba(0, 0, 0, 0.8);
  background: rgba(8, 11, 20, 0.78);
  border: 1px solid rgba(232, 234, 240, 0.14);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.42);
  backdrop-filter: blur(2px);
  padding: 3px 9px;
  border-radius: 6px;
}

.pin-tag {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #16a34a;
  background: rgba(10, 16, 12, 0.92);
  border: 1px solid rgba(22, 163, 74, 0.5);
  border-radius: 999px;
  padding: 2px 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
  pointer-events: none;
}
</style>
