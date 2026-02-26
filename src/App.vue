<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import GraphScene from '@/components/scene/GraphScene.vue'

const graphStore = useGraphStore()
onMounted(() => graphStore.initialize())
</script>

<template>
  <TresCanvas clear-color="#08080f" window-size>
    <GraphScene />
  </TresCanvas>
</template>
