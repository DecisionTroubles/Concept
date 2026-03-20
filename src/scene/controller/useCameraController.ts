import * as THREE from 'three'

export interface CameraFocusRequest {
  target: THREE.Vector3
  position: THREE.Vector3
}

export interface OrbitControlsLike {
  target: THREE.Vector3
  object: THREE.PerspectiveCamera
  update: () => void
}

export function useCameraController() {
  let focusRequest: CameraFocusRequest | null = null

  function requestFocus(request: CameraFocusRequest | null) {
    focusRequest = request
  }

  function clearFocusRequest() {
    focusRequest = null
  }

  function consumeFrame(camera: THREE.PerspectiveCamera, controls: OrbitControlsLike) {
    if (!focusRequest) return
    camera.position.lerp(focusRequest.position, 0.06)
    controls.target.lerp(focusRequest.target, 0.06)
    controls.update()
    if (camera.position.distanceTo(focusRequest.position) < 0.2) {
      focusRequest = null
    }
  }

  return {
    requestFocus,
    clearFocusRequest,
    consumeFrame,
  }
}
