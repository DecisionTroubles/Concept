<script setup lang="ts">
// TauRPC usage example — bindings.ts is generated on first `pnpm tauri dev` run.
// Once generated, import like this:
//
//   import { createTauRPCProxy } from './bindings'
//   const taurpc = createTauRPCProxy()
//   const msg = await taurpc.hello_world()

const msg = ref('')

async function callBackend() {
  // Placeholder until bindings.ts is generated
  const { createTauRPCProxy } = await import('../bindings')
  const taurpc = createTauRPCProxy()
  msg.value = await taurpc.hello_world()
}
</script>

<template>
  <div>
    <button @click="callBackend">Test TauRPC</button>
    <p v-if="msg">{{ msg }}</p>
  </div>
</template>
