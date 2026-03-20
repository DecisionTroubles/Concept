export interface CompassDotModel {
  id: string
  title: string
  screenX: number
  screenY: number
  edgeType: string
  index: number
}

export interface CompassHudModel {
  dots: CompassDotModel[]
  center: { x: number; y: number } | null
  neighborOrder: string[]
  activeIndex: number
}
