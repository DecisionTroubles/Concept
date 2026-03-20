<script setup lang="ts">
import { Html, Line2, OrbitControls } from '@tresjs/cientos'
import { useTres } from '@tresjs/core'
import { useEventListener, useRafFn } from '@vueuse/core'
import * as THREE from 'three'
import { computed, shallowRef, watch } from 'vue'
import type { SceneSnapshot, SceneSnapshotNode } from '@/scene/model/sceneSnapshot'
import type { useCameraController } from '@/scene/controller/useCameraController'
import { useSceneHudState } from '@/scene/controller/useSceneHudState'
import { projectHud } from '@/scene/projection/hudProjection'

const props = defineProps<{
  snapshot: SceneSnapshot
  cameraController: ReturnType<typeof useCameraController>
  activeKeys: Set<string>
}>()

const emit = defineEmits<{
  'node-clicked': [nodeId: string]
  'node-hovered': [nodeId: string | null]
  'background-clicked': []
}>()

const controlsRef = shallowRef()
const coreLightRef = shallowRef<THREE.PointLight | null>(null)
const settings = useSettings()
const editorMode = useEditorMode()
const hudState = useSceneHudState()

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const tres = useTres() as any

const cameraPos = new THREE.Vector3()
let pulseT = 0

const activeIndex = computed(() => hudState.activeIndex.value)

function nodeGeometryDetail(): number {
  return settings.graphics.nodeDetail
}

function sphereSegments(): [number, number] {
  const detail = nodeGeometryDetail()
  if (detail <= 0) return [12, 9]
  if (detail === 1) return [18, 14]
  return [24, 18]
}

function torusSegments(): [number, number] {
  const detail = nodeGeometryDetail()
  if (detail <= 0) return [8, 16]
  if (detail === 1) return [12, 24]
  return [16, 32]
}

function haloSegments(): [number, number] {
  const detail = nodeGeometryDetail()
  if (detail <= 0) return [10, 8]
  if (detail === 1) return [14, 10]
  return [18, 14]
}

function nodeLabelOpacity(node: SceneSnapshotNode): number {
  if (node.hovered || node.selected) return 1
  const dx = cameraPos.x - node.x
  const dy = cameraPos.y - node.y
  const dz = cameraPos.z - node.z
  const dist = Math.sqrt(dx * dx + dy * dy + dz * dz)
  if (node.labelPriority === 'high') {
    if (dist <= 20) return 1
    if (dist >= 78) return 0
    return 1 - (dist - 20) / 58
  }
  if (dist <= 14) return 1
  if (dist >= 42) return 0
  return 1 - (dist - 14) / 28
}

function nodeProgressLabel(node: SceneSnapshotNode): string {
  switch (node.progressStatus) {
    case 'learning':
      return 'Learning'
    case 'review':
      return 'Review'
    case 'mastered':
      return 'Mastered'
    default:
      return 'New'
  }
}

watch(
  () => settings.graphics.fogDensity,
  density => {
    try {
      const scene: THREE.Scene | undefined = tres?.scene?.value ?? tres?.scene
      if (scene?.fog instanceof THREE.FogExp2) scene.fog.density = density
    } catch {
      // ignore early setup races
    }
  },
  { immediate: true }
)

useEventListener(window, 'pointerdown', () => {
  props.cameraController.clearFocusRequest()
})

useRafFn(({ delta }) => {
  const raw = controlsRef.value
  const controls = raw?.instance ?? raw
  if (!controls?.object) return
  const camera = controls.object as THREE.PerspectiveCamera
  const speed = 14 * (delta / 1000)

  if (editorMode.mode.value === 'fly' && props.activeKeys.size > 0) {
    const forward = new THREE.Vector3()
    const right = new THREE.Vector3()
    const move = new THREE.Vector3()
    const up = new THREE.Vector3(0, 1, 0)

    camera.getWorldDirection(forward)
    forward.y = 0
    forward.normalize()
    right.crossVectors(forward, up).normalize()

    if (props.activeKeys.has(settings.keys.flyForward)) move.addScaledVector(forward, speed)
    if (props.activeKeys.has(settings.keys.flyBack)) move.addScaledVector(forward, -speed)
    if (props.activeKeys.has(settings.keys.flyLeft)) move.addScaledVector(right, -speed)
    if (props.activeKeys.has(settings.keys.flyRight)) move.addScaledVector(right, speed)
    if (props.activeKeys.has(settings.keys.flyDown)) move.y -= speed
    if (props.activeKeys.has(settings.keys.flyUp)) move.y += speed

    camera.position.add(move)
    controls.target.add(move)
    controls.update()
    props.cameraController.clearFocusRequest()
  }

  props.cameraController.consumeFrame(camera, controls)

  if (coreLightRef.value) {
    pulseT += delta / 1000
    coreLightRef.value.intensity = 55 + 22 * Math.sin(pulseT * 0.4)
  }

  cameraPos.copy(camera.position)

  if (editorMode.mode.value !== 'fly' && props.snapshot.activeNodeId) {
    hudState.updateHud(projectHud(props.snapshot, camera))
  } else {
    hudState.clearHud()
  }
})
</script>

<template>
  <TresPerspectiveCamera :position="[0, 8, 28]" :fov="60" />
  <OrbitControls ref="controlsRef" enable-damping :damping-factor="0.05" />

  <TresHemisphereLight sky-color="#2a3a66" ground-color="#08080f" :intensity="0.7" />
  <TresDirectionalLight :position="[12, 20, 8]" color="#ccd8ff" :intensity="1.8" />
  <TresPointLight ref="coreLightRef" :position="[0, 0, 0]" color="#4466ee" :intensity="55" :distance="42" :decay="1.8" />
  <TresPointLight :position="[0, 16, 0]" color="#5566ff" :intensity="40" :distance="50" :decay="1.5" />
  <TresPointLight :position="[-14, -7, 12]" color="#ff3366" :intensity="28" :distance="40" :decay="2" />
  <TresPointLight :position="[14, -5, -10]" color="#33aaff" :intensity="18" :distance="35" :decay="2" />

  <Line2
    v-for="edge in snapshot.edges"
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

  <TresGroup>
    <TresMesh
      v-for="node in snapshot.nodes"
      :key="node.id"
      :position="[node.x, node.y, node.z]"
      :scale="node.scale"
      @click="($event: any) => { $event.stopPropagation?.(); emit('node-clicked', node.id) }"
      @pointer-enter="($event: any) => { $event.stopPropagation?.(); emit('node-hovered', node.id) }"
      @pointer-leave="($event: any) => { $event.stopPropagation?.(); emit('node-hovered', null) }"
    >
      <TresOctahedronGeometry v-if="node.node_type === 'grammar'" :args="[node.radius * 0.85, nodeGeometryDetail()]" />
      <TresBoxGeometry
        v-else-if="node.node_type === 'kanji'"
        :args="[node.radius * 1.2, node.radius * 1.2, node.radius * 1.2]"
      />
      <TresIcosahedronGeometry v-else-if="node.node_type === 'concept'" :args="[node.radius * 0.9, nodeGeometryDetail()]" />
      <TresTorusGeometry
        v-else-if="node.node_type === 'particle'"
        :args="[node.radius * 0.7, node.radius * 0.22, torusSegments()[0], torusSegments()[1]]"
      />
      <TresSphereGeometry v-else :args="[node.radius, sphereSegments()[0], sphereSegments()[1]]" />
      <TresMeshStandardMaterial
        :color="node.color"
        :emissive="node.emissive"
        :emissive-intensity="node.emissiveIntensity"
        :roughness="0.35"
        :metalness="0.4"
      />
    </TresMesh>

    <TresMesh
      v-for="node in snapshot.nodes"
      :key="`halo-${node.id}`"
      :position="[node.x, node.y, node.z]"
      :scale="node.scale * 1.05"
    >
      <TresSphereGeometry :args="[node.radius * 1.8, haloSegments()[0], haloSegments()[1]]" />
      <TresMeshBasicMaterial
        :color="node.color"
        :opacity="node.neighbor || node.selected ? 0.16 : 0.1"
        transparent
        :depth-write="false"
      />
    </TresMesh>
  </TresGroup>

  <Html
    v-for="node in snapshot.nodes"
    :key="`label-${node.id}`"
    :position="[node.x, node.y + node.radius + 0.7, node.z]"
    center
    :sprite="true"
    :z-index-range="[40, 0]"
  >
    <div v-if="nodeLabelOpacity(node) > 0.01" class="node-label" :style="{ opacity: nodeLabelOpacity(node) }">
      <span>{{ node.title }}</span>
      <span class="node-progress-chip" :class="`is-${node.progressStatus}`" :title="`Progress: ${nodeProgressLabel(node)}`">
        {{ nodeProgressLabel(node) }}
      </span>
    </div>
  </Html>

  <Html
    v-for="node in snapshot.nodes.filter(item => item.pinned)"
    :key="`pin-tag-${node.id}`"
    :position="[node.x, node.y + node.radius + 1.3, node.z]"
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
  display: inline-flex;
  align-items: center;
  gap: 6px;
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

.node-progress-chip {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 2px 7px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  border: 1px solid transparent;
}

.node-progress-chip.is-new {
  color: #7dd3fc;
  background: rgba(14, 165, 233, 0.14);
  border-color: rgba(14, 165, 233, 0.24);
}

.node-progress-chip.is-learning {
  color: #fbbf24;
  background: rgba(245, 158, 11, 0.14);
  border-color: rgba(245, 158, 11, 0.24);
}

.node-progress-chip.is-review {
  color: #c084fc;
  background: rgba(168, 85, 247, 0.14);
  border-color: rgba(168, 85, 247, 0.24);
}

.node-progress-chip.is-mastered {
  color: #4ade80;
  background: rgba(34, 197, 94, 0.14);
  border-color: rgba(34, 197, 94, 0.24);
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
