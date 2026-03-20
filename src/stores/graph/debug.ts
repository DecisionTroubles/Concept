declare global {
  interface Window {
    __GRAPH_TRACE__?: boolean
    __GRAPH_TRACE_BUFFER__?: Array<{ seq: number; event: string; payload?: Record<string, unknown>; at: string }>
    __dumpGraphTrace__?: () => Array<{ seq: number; event: string; payload?: Record<string, unknown>; at: string }>
  }
}

let traceSequence = 0
const TRACE_BUFFER_LIMIT = 80

type GraphTraceEntry = {
  seq: number
  event: string
  payload?: Record<string, unknown>
  at: string
}

function tracingEnabled(): boolean {
  if (typeof window === 'undefined') return false
  return window.__GRAPH_TRACE__ === true
}

function pushTraceEntry(entry: GraphTraceEntry) {
  if (typeof window === 'undefined') return
  const buffer = window.__GRAPH_TRACE_BUFFER__ ?? []
  buffer.push(entry)
  if (buffer.length > TRACE_BUFFER_LIMIT) {
    buffer.splice(0, buffer.length - TRACE_BUFFER_LIMIT)
  }
  window.__GRAPH_TRACE_BUFFER__ = buffer
  window.__dumpGraphTrace__ = () => [...buffer]
}

export function graphTrace(event: string, payload?: Record<string, unknown>) {
  traceSequence += 1
  const entry: GraphTraceEntry = {
    seq: traceSequence,
    event,
    payload,
    at: new Date().toISOString(),
  }
  pushTraceEntry(entry)
  if (!tracingEnabled()) return
  if (payload) console.log(`[graph-trace #${traceSequence}] ${event}`, payload)
  else console.log(`[graph-trace #${traceSequence}] ${event}`)
}

export function getGraphTraceBuffer(): Array<{ seq: number; event: string; payload?: Record<string, unknown>; at: string }> {
  if (typeof window === 'undefined') return []
  return [...(window.__GRAPH_TRACE_BUFFER__ ?? [])]
}
