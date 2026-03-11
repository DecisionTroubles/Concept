declare global {
  interface Window {
    __GRAPH_TRACE__?: boolean
  }
}

let traceSequence = 0

function tracingEnabled(): boolean {
  if (typeof window === 'undefined') return false
  return window.__GRAPH_TRACE__ === true
}

export function graphTrace(event: string, payload?: Record<string, unknown>) {
  if (!tracingEnabled()) return
  traceSequence += 1
  if (payload) console.log(`[graph-trace #${traceSequence}] ${event}`, payload)
  else console.log(`[graph-trace #${traceSequence}] ${event}`)
}
