import { useEventListener } from '@vueuse/core'
import type { useSceneController } from '@/scene/controller/useSceneController'

interface SceneInputRouterOptions {
  controller: ReturnType<typeof useSceneController>
}

export function useSceneInputRouter(options: SceneInputRouterOptions) {
  const graphStore = useGraphStore()
  const editorMode = useEditorMode()
  const settings = useSettings()

  const activeKeys = new Set<string>()

  function isInteractiveUiTarget(target: EventTarget | null): boolean {
    const element = target instanceof HTMLElement ? target : null
    if (!element) return false
    if (element.isContentEditable) return true
    return Boolean(
      element.closest(
        [
          'input',
          'textarea',
          'select',
          'button',
          'a[href]',
          '[role="button"]',
          '[role="textbox"]',
          '[contenteditable="true"]',
          '[data-hotkey-input="true"]',
        ].join(', ')
      )
    )
  }

  useEventListener(window, 'keydown', event => {
    const isInteractiveTarget = isInteractiveUiTarget(event.target)
    const key = event.key.toLowerCase()
    const isSpaceFocusKey = event.key === ' '

    if (!isInteractiveTarget && key === settings.keys.pinnedBuffer) {
      event.preventDefault()
      graphStore.openBuffer('pinned')
      return
    }
    if (!isInteractiveTarget && key === settings.keys.packsBuffer) {
      event.preventDefault()
      graphStore.openPackLibrary()
      return
    }
    if (!isInteractiveTarget && key === settings.keys.mapBuffer) {
      event.preventDefault()
      graphStore.openBuffer('map')
      return
    }
    if (!isInteractiveTarget && event.key === 'Escape' && graphStore.activeBuffer !== 'none') {
      event.preventDefault()
      graphStore.closeBuffer()
      return
    }

    if (isInteractiveTarget) return

    if (graphStore.activeBuffer !== 'none') return

    if (editorMode.mode.value === 'fly') {
      const flyMoveKeys = [
        settings.keys.flyForward,
        settings.keys.flyBack,
        settings.keys.flyLeft,
        settings.keys.flyRight,
        settings.keys.flyUp,
        settings.keys.flyDown,
      ]
      if (flyMoveKeys.includes(key)) {
        activeKeys.add(key)
        event.preventDefault()
      }
    }

    if (event.key === 'Escape') {
      event.preventDefault()
      options.controller.handleEscape()
      return
    }

    if (key === settings.keys.flyMode) {
      editorMode.enterFly()
      return
    }
    if (key === settings.keys.graphMode && graphStore.selectedNodeId) {
      editorMode.enterGraph()
      return
    }
    if (graphStore.selectedNodeId && (key === settings.keys.openNode || event.key === 'Enter')) {
      event.preventDefault()
      options.controller.toggleCenteredDetail()
      return
    }
    if (graphStore.selectedNodeId && key === settings.keys.pinNode) {
      event.preventDefault()
      options.controller.togglePin(graphStore.selectedNodeId)
      return
    }
    if (graphStore.selectedNodeId && (key === settings.keys.focusView || isSpaceFocusKey)) {
      event.preventDefault()
      options.controller.toggleSolar(graphStore.selectedNodeId)
      return
    }

    if (
      graphStore.focusViewActive &&
      (event.key === 'ArrowLeft' || event.key === 'ArrowRight' || event.key === 'ArrowUp' || event.key === 'ArrowDown')
    ) {
      event.preventDefault()
      options.controller.moveSolarCursor(
        event.key === 'ArrowLeft'
          ? 'left'
          : event.key === 'ArrowRight'
            ? 'right'
            : event.key === 'ArrowUp'
              ? 'up'
              : 'down'
      )
      return
    }

    if (graphStore.selectedNodeId && editorMode.mode.value !== 'fly' && !graphStore.centeredNodePanel) {
      if (event.key === 'Tab') {
        event.preventDefault()
        options.controller.cycleNeighbor(!event.shiftKey)
        return
      }
      const num = parseInt(event.key, 10)
      if (num >= 1 && num <= 9) {
        event.preventDefault()
        options.controller.jumpToNeighbor(num)
        return
      }
    }

    if (key === settings.keys.jumpBack && editorMode.mode.value !== 'fly') {
      event.preventDefault()
      options.controller.jumpBack()
    }
  })

  useEventListener(window, 'keyup', event => {
    activeKeys.delete(event.key.toLowerCase())
  })

  return {
    activeKeys,
  }
}
