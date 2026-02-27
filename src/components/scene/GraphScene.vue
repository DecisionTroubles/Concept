<script setup lang="ts">
import { Html, Line2, OrbitControls } from '@tresjs/cientos'
import { useTres } from '@tresjs/core'
import { useRafFn } from '@vueuse/core'
import * as THREE from 'three'
import { computed, onMounted, ref, shallowRef, watch } from 'vue'
import { useForceLayout, type PositionedNode } from '@/composables/useForceLayout'

const graphStore = useGraphStore()
const controlsRef  = shallowRef()
const coreLightRef = shallowRef<THREE.PointLight | null>(null)

// TresJS context — used for fog setup.
// Accessed as `any` since the exact shape of useTres() varies across versions.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const tres = useTres() as any

// ── WASD + Q/E movement ───────────────────────────────────────────────────────
const keys = { w: false, a: false, s: false, d: false, q: false, e: false }

onMounted(() => {
  // Keyboard listeners
  window.addEventListener('keydown', (e) => {
    const k = e.key.toLowerCase()
    if (k in keys) (keys as Record<string, boolean>)[k] = true
    if (e.key === 'Escape') graphStore.selectNode(null)
  })
  window.addEventListener('keyup', (e) => {
    const k = e.key.toLowerCase()
    if (k in keys) (keys as Record<string, boolean>)[k] = false
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

// Pulse state for the core light — lives outside Vue reactivity to avoid
// per-frame re-render cost.
let pulseT = 0

useRafFn(({ delta }) => {
  const raw = controlsRef.value
  const controls = raw?.instance ?? raw
  if (!controls?.object) return

  const cam   = controls.object as THREE.PerspectiveCamera
  const speed = 14 * (delta / 1000)

  // WASD flythrough
  if (keys.w || keys.a || keys.s || keys.d || keys.q || keys.e) {
    cam.getWorldDirection(_fwd)
    _fwd.y = 0
    _fwd.normalize()
    _right.crossVectors(_fwd, _up).normalize()
    _move.set(0, 0, 0)

    if (keys.w) _move.addScaledVector(_fwd,    speed)
    if (keys.s) _move.addScaledVector(_fwd,   -speed)
    if (keys.a) _move.addScaledVector(_right, -speed)
    if (keys.d) _move.addScaledVector(_right,  speed)
    if (keys.q) _move.y -= speed
    if (keys.e) _move.y += speed

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

  // Animate central core light — bypasses Vue reactivity for zero overhead.
  if (coreLightRef.value) {
    pulseT += delta / 1000
    coreLightRef.value.intensity = 55 + 22 * Math.sin(pulseT * 0.4)
  }
})

// ── Hover state ───────────────────────────────────────────────────────────────
const hoveredNodeId = ref<string | null>(null)

// ── Node helpers ──────────────────────────────────────────────────────────────
function nodeRadius(node: PositionedNode): number {
  // Larger base radius, wider range so weight is more visible.
  return Math.min(1.4, Math.max(0.55, 0.65 + (node.weight ?? 1) * 0.2))
}

function nodeColor(node: PositionedNode): string {
  if (graphStore.selectedNodeId === node.id) return '#ffffff'
  if (node.learned) return '#3dd68c'
  return '#4a5080'
}

function nodeEmissive(node: PositionedNode): string {
  if (graphStore.selectedNodeId === node.id) return '#5555cc'
  if (node.learned) return '#1a6644'
  return '#0a0a1a'
}

function nodeEmissiveIntensity(node: PositionedNode): number {
  if (graphStore.selectedNodeId === node.id) return 1.5
  if (node.learned) return 0.8
  return 0.15
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

// ── Event handlers ────────────────────────────────────────────────────────────
function onNodeClick(node: PositionedNode, event: { stopPropagation?: () => void }) {
  event.stopPropagation?.()
  graphStore.selectNode(node.id)
  focusTarget.value = new THREE.Vector3(node.x, node.y, node.z)
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

// Only mount Html label elements for the hovered / selected node.
const labelNodes = computed(() =>
  positionedNodes.value.filter(
    (n) => hoveredNodeId.value === n.id || graphStore.selectedNodeId === n.id,
  ),
)
</script>

<template>
  <!-- Camera -->
  <TresPerspectiveCamera :position="[0, 8, 28]" :fov="60" />
  <OrbitControls ref="controlsRef" enable-damping :damping-factor="0.05" />

  <!-- ── Lighting ─────────────────────────────────────────────────────────── -->
  <!--
    Hemisphere light: sky = cool blue-purple, ground = near black.
    Gives a natural sky/ground gradient instead of flat uniform ambient.
  -->
  <TresHemisphereLight sky-color="#2a3a66" ground-color="#08080f" :intensity="0.7" />

  <!-- Directional light — main "sun" from upper right -->
  <TresDirectionalLight :position="[12, 20, 8]" color="#ccd8ff" :intensity="1.8" />

  <!--
    Central core light — pulses slowly via direct Three.js mutation in RAF.
    ref gives us the raw PointLight instance without going through Vue reactivity.
  -->
  <TresPointLight
    ref="coreLightRef"
    :position="[0, 0, 0]"
    color="#4466ee"
    :intensity="55"
    :distance="42"
    :decay="1.8"
  />

  <!-- Accent fill lights -->
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
    <!-- Geometry by node type -->
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
    <!-- Default: sphere (vocabulary, root, etc.) -->
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

  <!-- ── Node labels (mounted only for hovered / selected) ────────────────── -->
  <Html
    v-for="node in labelNodes"
    :key="`label-${node.id}`"
    :position="[node.x, node.y + nodeRadius(node) + 0.7, node.z]"
    center
    :sprite="true"
  >
    <div class="node-label">{{ node.title }}</div>
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
