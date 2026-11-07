export type Point = { x: number; y: number };
export function rectangle(a: Point, b: Point) {
  return {
    left: Math.min(a.x, b.x),
    top: Math.min(a.y, b.y),
    width: Math.abs(a.x - b.x),
    height: Math.abs(a.y - b.y),
  };
}
export function clampPoint(
  x: number,
  y: number,
  width: number,
  height: number,
): Point {
  return {
    x: Math.max(0, Math.min(x, width)),
    y: Math.max(0, Math.min(y, height)),
  };
}
