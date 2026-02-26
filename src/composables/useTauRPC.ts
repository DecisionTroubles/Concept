import { createTauRPCProxy } from '@/bindings'

let _proxy: ReturnType<typeof createTauRPCProxy> | null = null

export function useTauRPC() {
  if (!_proxy) {
    _proxy = createTauRPCProxy()
  }
  return _proxy
}
