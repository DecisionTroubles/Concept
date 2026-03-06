Drop `.ts`, `.js`, or `.mjs` plugin modules in this folder.

They are loaded automatically after core plugins.

Supported exports:

```ts
export default definePlugin({...})
export const plugin = definePlugin({...})
export const plugins = [definePlugin({...})]
```

See `example.plugin.ts` for the smallest working shape.
