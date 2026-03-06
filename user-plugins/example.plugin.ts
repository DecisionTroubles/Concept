import { definePlugin } from '@/core/plugin'

export default definePlugin({
  id: 'user.example',
  name: 'User Example',
  nodeWorkspaceExtensions: [
    {
      id: 'user.example.extension',
      title: 'User Extension',
      description: 'Example user-provided node workspace block.',
      slot: 'extensions.primary',
      order: 100,
    },
  ],
})
