export function quaternionToYaw(y: number, w: number): number {
  const yaw = Math.atan2(2 * y * w, 1 - 2 * y * y);
  return yaw;
}
