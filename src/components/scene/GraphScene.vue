<script setup lang="ts">
import { OrbitControls } from '@tresjs/cientos'
import { useRafFn } from '@vueuse/core'
import * as THREE from 'three'

const graphStore = useGraphStore()
const controlsRef = shallowRef()

// ── WASD + Q/E movement ───────────────────────────────────────────────────────
const keys = { w: false, a: false, s: false, d: false, q: false, e: false }

onMounted(() => {
  window.addEventListener('keydown', (e) => {
    const k = e.key.toLowerCase()
    if (k in keys) (keys as any)[k] = true
  })
  window.addEventListener('keyup', (e) => {
    const k = e.key.toLowerCase()
    if (k in keys) (keys as any)[k] = false
  })
})

const _fwd   = new THREE.Vector3()
const _right = new THREE.Vector3()
const _move  = new THREE.Vector3()
const _up    = new THREE.Vector3(0, 1, 0)

useRafFn(({ delta }) => {
  // cientos exposes the Three.js OrbitControls via 'instance' or directly
  const raw = controlsRef.value
  const controls = raw?.instance ?? raw
  if (!controls?.object) return
  if (!keys.w && !keys.a && !keys.s && !keys.d && !keys.q && !keys.e) return

  const cam   = controls.object as THREE.PerspectiveCamera
  const speed = 14 * (delta / 1000)   // delta is ms from vueuse

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
})

// ── Node layout ───────────────────────────────────────────────────────────────
const positionedNodes = computed(() => {
  const n = graphStore.nodes.length
  if (n === 0) return []
  return graphStore.nodes.map((node, i) => ({
    ...node,
    x: node.pos_x ?? 8 * Math.cos((2 * Math.PI * i) / n),
    y: node.pos_y ?? 0,
    z: node.pos_z ?? 8 * Math.sin((2 * Math.PI * i) / n),
  }))
})
</script>

<template>
  <!-- Camera -->
  <TresPerspectiveCamera :position="[0, 8, 28]" :fov="60" />
  <OrbitControls ref="controlsRef" enable-damping :damping-factor="0.05" />

  <!-- Lighting -->
  <TresAmbientLight color="#aabbff" :intensity="0.5" />
  <TresDirectionalLight :position="[12, 20, 8]" color="#ffffff" :intensity="2" />
  <TresPointLight :position="[0, 12, 0]"    color="#6677ff" :intensity="60" :distance="40" />
  <TresPointLight :position="[-12, -6, 10]" color="#ff4488" :intensity="30" :distance="35" />

  <!-- Graph nodes -->
  <TresMesh
    v-for="node in positionedNodes"
    :key="node.id"
    :position="[node.x, node.y, node.z]"
  >
    <TresSphereGeometry :args="[0.6, 32, 32]" />
    <TresMeshStandardMaterial
      :color="node.learned ? '#44ff88' : '#6644ff'"
      emissive="#221133"
      :emissive-intensity="0.6"
    />
  </TresMesh>
</template>
