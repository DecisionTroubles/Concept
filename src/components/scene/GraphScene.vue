<script setup lang="ts">
import { Html, Line2, OrbitControls } from '@tresjs/cientos'
import { useTres } from '@tresjs/core'
import { useRafFn } from '@vueuse/core'
import * as THREE from 'three'
import { computed, onMounted, shallowRef, watch, ref } from 'vue'
import { useForceLayout, type PositionedNode } from '@/composables/useForceLayout'
import { COMPASS_RING_R, type CompassDot } from '@/composables/useEditorMode'

const graphStore = useGraphStore()
const controlsRef  = shallowRef()
const coreLightRef = shallowRef<THREE.PointLight | null>(null)

// TresJS context — used for fog setup.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const tres = useTres() as any

// ── Editor mode ───────────────────────────────────────────────────────────────
const editorMode = useEditorMode()

// ── Settings (configurable keybindings) ───────────────────────────────────────
const settings = useSettings()

// ── Fly key tracking (only active in fly mode) ────────────────────────────────
const activeKeys = new Set<string>()

// Sync mode when node deselected from outside (X button, layer switch, etc.)
watch(() => graphStore.selectedNodeId, (id) => editorMode.onNodeSelected(id))

// Clear stuck fly keys when leaving fly mode
watch(() => editorMode.mode.value, m => { if (m !== 'fly') activeKeys.clear() })

// IDs of nodes directly connected to the selected node
const neighborIds = computed<Set<string>>(() => {
  const sel = graphStore.selectedNode
  if (!sel) return new Set()
  return new Set(sel.connections.map(c => c.target_id))
})

// Focus camera when search requests focus (even for the same node re-selected)
watch(() => graphStore.focusVersion, () => {
  const id = graphStore.selectedNodeId
  if (id) {
    const t = positionedNodes.value.find(n => n.id === id)
    if (t) focusTarget.value = new THREE.Vector3(t.x, t.y, t.z)
  }
})

onMounted(() => {
  window.addEventListener('keydown', (e) => {
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA'
                  || (e.target as HTMLElement)?.isContentEditable

    // Fly key tracking
    if (editorMode.mode.value === 'fly') {
      const k = e.key.toLowerCase()
      const flyMoveKeys = [
        settings.keys.flyForward, settings.keys.flyBack,
        settings.keys.flyLeft,    settings.keys.flyRight,
        settings.keys.flyUp,      settings.keys.flyDown,
      ]
      if (flyMoveKeys.includes(k)) { activeKeys.add(k); e.preventDefault() }
    }

    if (e.key === 'Escape') { e.preventDefault(); editorMode.escapeFromCurrentMode(); return }
    if (!isInput && e.key.toLowerCase() === settings.keys.flyMode) { editorMode.enterFly(); return }
    if (!isInput && e.key.toLowerCase() === settings.keys.graphMode && graphStore.selectedNodeId) {
      editorMode.enterGraph(); return
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
    if (!isInput && e.key.toLowerCase() === settings.keys.jumpBack && editorMode.mode.value !== 'fly') {
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

  window.addEventListener('keyup', (e) => {
    activeKeys.delete(e.key.toLowerCase())
  })

  // Scene fog — gives the graph depth and a sense of infinite space.
  try {
    const scene: THREE.Scene | undefined = tres?.scene?.value ?? tres?.scene
    if (scene instanceof THREE.Scene) {
      scene.fog = new THREE.FogExp2(new THREE.Color('#080b14'), 0.014)
    }
  } catch { /* skip if context not yet ready */ }
})

// ── Force layout ──────────────────────────────────────────────────────────────
const { positionedNodes } = useForceLayout(
  computed(() => graphStore.nodes),
  (settled) => {
    for (const node of settled) {
      graphStore.updateNodePosition(node.id, node.x, node.y, node.z)
    }
  },
)

// ── Camera focus animation ────────────────────────────────────────────────────
const focusTarget = shallowRef<THREE.Vector3 | null>(null)

const _fwd            = new THREE.Vector3()
const _right          = new THREE.Vector3()
const _move           = new THREE.Vector3()
const _up             = new THREE.Vector3(0, 1, 0)
const _camLerpTarget  = new THREE.Vector3()
const _orbitLerpTarget = new THREE.Vector3()

// Pulse state for the core light
let pulseT = 0


// Camera position for distance-faded labels
const cameraPos = shallowRef(new THREE.Vector3())

const _ndcVec = new THREE.Vector3()

useRafFn(({ delta }) => {
  const raw = controlsRef.value
  const controls = raw?.instance ?? raw
  if (!controls?.object) return

  const cam   = controls.object as THREE.PerspectiveCamera
  const speed = 14 * (delta / 1000)

  // Fly mode movement — only when in fly mode and keys are held
  if (editorMode.mode.value === 'fly' && activeKeys.size > 0) {
    cam.getWorldDirection(_fwd)
    _fwd.y = 0
    _fwd.normalize()
    _right.crossVectors(_fwd, _up).normalize()
    _move.set(0, 0, 0)

    if (activeKeys.has(settings.keys.flyForward)) _move.addScaledVector(_fwd,    speed)
    if (activeKeys.has(settings.keys.flyBack))    _move.addScaledVector(_fwd,   -speed)
    if (activeKeys.has(settings.keys.flyLeft))    _move.addScaledVector(_right, -speed)
    if (activeKeys.has(settings.keys.flyRight))   _move.addScaledVector(_right,  speed)
    if (activeKeys.has(settings.keys.flyDown))    _move.y -= speed
    if (activeKeys.has(settings.keys.flyUp))      _move.y += speed

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


  // Track camera position for distance-faded labels
  cameraPos.value = cam.position.clone()

  // Compass projection (graph mode only)
  if (editorMode.mode.value === 'graph' && graphStore.selectedNodeId) {
    const sel = positionedNodes.value.find(n => n.id === graphStore.selectedNodeId)
    if (sel) {
      _ndcVec.set(sel.x, sel.y, sel.z).project(cam)
      const sx = (_ndcVec.x + 1) / 2 * window.innerWidth
      const sy = (-_ndcVec.y + 1) / 2 * window.innerHeight
      const center = { x: sx, y: sy }
      const nodeMap = new Map(positionedNodes.value.map(n => [n.id, n]))
      // Deduplicate by target_id (same neighbor can appear via both an outgoing
      // and an incoming edge after the bidirectional query), then filter to
      // nodes present in this layer, so indices are always sequential 1, 2, 3…
      const seen = new Set<string>()
      const validConns = sel.connections
        .filter(conn => {
          if (!nodeMap.has(conn.target_id) || seen.has(conn.target_id)) return false
          seen.add(conn.target_id)
          return true
        })
        .slice(0, 9)
      const dots: CompassDot[] = validConns.map((conn, i) => {
        const nb = nodeMap.get(conn.target_id)!
        _ndcVec.set(nb.x, nb.y, nb.z).project(cam)
        const nx = (_ndcVec.x + 1) / 2 * window.innerWidth
        const ny = (-_ndcVec.y + 1) / 2 * window.innerHeight
        const angle = Math.atan2(ny - sy, nx - sx)
        return { id: conn.target_id, title: nb.title,
                 screenX: sx + Math.cos(angle) * COMPASS_RING_R,
                 screenY: sy + Math.sin(angle) * COMPASS_RING_R,
                 edgeType: conn.edge_type, index: i + 1 }
      })
      editorMode.setNeighborOrder(dots.map(d => d.id))
      editorMode.setCompassState(dots, center)
    }
  } else {
    editorMode.setNeighborOrder([])
    editorMode.setCompassState([], null)
  }
})

// ── Hover state ───────────────────────────────────────────────────────────────
const hoveredNodeId = ref<string | null>(null)

// ── Node helpers ──────────────────────────────────────────────────────────────
function nodeRadius(node: PositionedNode): number {
  return Math.min(1.4, Math.max(0.55, 0.65 + (node.weight ?? 1) * 0.2))
}

// Per-type base colors (saturated, readable against dark bg)
const TYPE_COLORS: Record<string, string> = {
  grammar:  '#5b7fe0',
  kanji:    '#d4872a',
  vocab:    '#3bbf70',
  particle: '#c24060',
  concept:  '#9060cc',
  root:     '#7888aa',
}

// Per-type emissive glow
const TYPE_EMISSIVE: Record<string, string> = {
  grammar:  '#1a2580',
  kanji:    '#3d2000',
  vocab:    '#0f3d20',
  particle: '#3d0f1a',
  concept:  '#250a50',
  root:     '#1a2030',
}

function nodeColor(node: PositionedNode): string {
  if (graphStore.selectedNodeId === node.id) return '#ffffff'
  if (neighborIds.value.has(node.id)) return '#5ba8ff'
  if (node.learned) return '#3dd68c'
  return TYPE_COLORS[node.node_type] ?? '#5870a0'
}

function nodeEmissive(node: PositionedNode): string {
  if (graphStore.selectedNodeId === node.id) return '#5555cc'
  if (neighborIds.value.has(node.id)) return '#1a4aee'
  if (node.learned) return '#1a6644'
  return TYPE_EMISSIVE[node.node_type] ?? '#0f1556'
}

function nodeEmissiveIntensity(node: PositionedNode): number {
  if (graphStore.selectedNodeId === node.id) return 1.5
  if (neighborIds.value.has(node.id))
    return 0.45 + 0.4 * Math.sin(Date.now() / 400)
  if (node.learned) return 0.8
  return 0.7
}

function nodeScale(node: PositionedNode): number {
  return hoveredNodeId.value === node.id ? 1.25 : 1.0
}

// ── Edge helpers ──────────────────────────────────────────────────────────────
function edgeColor(edgeType: string): string {
  switch (edgeType) {
    case 'Prerequisite': return '#5b8fff'
    case 'Semantic':     return '#5a648c'
    case 'UserDefined':  return '#f59e0b'
    case 'Context':
    default:             return '#4a5068'
  }
}

function edgeLineWidth(edgeType: string): number {
  switch (edgeType) {
    case 'Prerequisite': return 2.5
    case 'Semantic':     return 1.2
    default:             return 1.8
  }
}

// ── Edge list for cientos Line2 component ────────────────────────────────────
const edges = computed(() => {
  const nodeMap = new Map<string, PositionedNode>(positionedNodes.value.map((n) => [n.id, n]))
  const result: {
    id: string
    points: [[number, number, number], [number, number, number]]
    color: string
    width: number
  }[] = []

  for (const node of positionedNodes.value) {
    for (const conn of node.connections) {
      const target = nodeMap.get(conn.target_id)
      if (!target) continue
      result.push({
        id: conn.id,
        points: [
          [node.x, node.y, node.z],
          [target.x, target.y, target.z],
        ],
        color: edgeColor(conn.edge_type),
        width: edgeLineWidth(conn.edge_type),
      })
    }
  }

  return result
})

// ── Distance-faded label opacity ──────────────────────────────────────────────
function nodeLabelOpacity(node: PositionedNode): number {
  if (hoveredNodeId.value === node.id || graphStore.selectedNodeId === node.id) return 1
  const dx = cameraPos.value.x - node.x
  const dy = cameraPos.value.y - node.y
  const dz = cameraPos.value.z - node.z
  const dist = Math.sqrt(dx*dx + dy*dy + dz*dz)
  if (dist <= 12) return 1
  if (dist >= 30) return 0
  return 1 - (dist - 12) / 18
}

// ── Event handlers ────────────────────────────────────────────────────────────
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

// Watch for layer changes — reset focus
watch(() => graphStore.activeLayerId, () => {
  focusTarget.value  = null
  hoveredNodeId.value = null
})
</script>

<template>
  <!-- Camera -->
  <TresPerspectiveCamera :position="[0, 8, 28]" :fov="60" />
  <OrbitControls ref="controlsRef" enable-damping :damping-factor="0.05" />

  <!-- ── Lighting ─────────────────────────────────────────────────────────── -->
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
  <TresPointLight :position="[0, 16, 0]"     color="#5566ff" :intensity="40" :distance="50" :decay="1.5" />
  <TresPointLight :position="[-14, -7, 12]"  color="#ff3366" :intensity="28" :distance="40" :decay="2" />
  <TresPointLight :position="[14, -5, -10]"  color="#33aaff" :intensity="18" :distance="35" :decay="2" />

  <!-- ── Edges ────────────────────────────────────────────────────────────── -->
  <Line2
    v-for="edge in edges"
    :key="edge.id"
    :points="edge.points"
    :color="edge.color"
    :line-width="edge.width"
  />

  <!-- ── Nodes ────────────────────────────────────────────────────────────── -->
  <TresMesh
    v-for="node in positionedNodes"
    :key="node.id"
    :position="[node.x, node.y, node.z]"
    :scale="nodeScale(node)"
    @click="(e: any) => onNodeClick(node, e)"
    @pointer-enter="(e: any) => onNodePointerEnter(node, e)"
    @pointer-leave="(e: any) => onNodePointerLeave(node, e)"
  >
    <TresOctahedronGeometry
      v-if="node.node_type === 'grammar'"
      :args="[nodeRadius(node) * 0.85, 0]"
    />
    <TresBoxGeometry
      v-else-if="node.node_type === 'kanji'"
      :args="[nodeRadius(node) * 1.2, nodeRadius(node) * 1.2, nodeRadius(node) * 1.2]"
    />
    <TresIcosahedronGeometry
      v-else-if="node.node_type === 'concept'"
      :args="[nodeRadius(node) * 0.9, 0]"
    />
    <TresTorusGeometry
      v-else-if="node.node_type === 'particle'"
      :args="[nodeRadius(node) * 0.7, nodeRadius(node) * 0.22, 12, 24]"
    />
    <TresSphereGeometry
      v-else
      :args="[nodeRadius(node), 32, 32]"
    />
    <TresMeshStandardMaterial
      :color="nodeColor(node)"
      :emissive="nodeEmissive(node)"
      :emissive-intensity="nodeEmissiveIntensity(node)"
      :roughness="0.35"
      :metalness="0.4"
    />
  </TresMesh>

  <!-- ── Node labels (all nodes, distance-faded) ───────────────────────────── -->
  <Html
    v-for="node in positionedNodes"
    :key="`label-${node.id}`"
    :position="[node.x, node.y + nodeRadius(node) + 0.7, node.z]"
    center
    :sprite="true"
  >
    <div class="node-label" :style="{ opacity: nodeLabelOpacity(node), transition: 'opacity 0.3s' }">
      {{ node.title }}
    </div>
  </Html>

</template>

<style scoped>
.node-label {
  color: #e8eaf0;
  font-family: system-ui, sans-serif;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.02em;
  white-space: nowrap;
  pointer-events: none;
  text-shadow:
    0 0 6px rgba(0, 0, 0, 0.95),
    0 0 14px rgba(0, 0, 0, 0.8);
  background: rgba(8, 11, 20, 0.6);
  padding: 2px 7px;
  border-radius: 4px;
}
</style>
