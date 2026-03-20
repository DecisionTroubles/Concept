export type GraphFocusState =
  | { mode: 'world' }
  | { mode: 'solar'; rootNodeId: string; cursorNodeId: string }

export function createWorldFocusState(): GraphFocusState {
  return { mode: 'world' }
}

export function createSolarFocusState(rootNodeId: string, cursorNodeId = rootNodeId): GraphFocusState {
  return { mode: 'solar', rootNodeId, cursorNodeId }
}

export function isSolarFocusState(state: GraphFocusState): state is Extract<GraphFocusState, { mode: 'solar' }> {
  return state.mode === 'solar'
}

export function focusRootNodeId(state: GraphFocusState): string | null {
  return isSolarFocusState(state) ? state.rootNodeId : null
}

export function focusCursorNodeId(state: GraphFocusState): string | null {
  return isSolarFocusState(state) ? state.cursorNodeId : null
}
