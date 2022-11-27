import { Quad, Triplet } from "engine";

export function eulerToQuaternion(euler: Triplet): Quad {
  const x = euler[0] * 0.5;
  const y = euler[1] * 0.5;
  const z = euler[2] * 0.5;

  const sx = Math.sin(x);
  const cx = Math.cos(x);
  const sy = Math.sin(y);
  const cy = Math.cos(y);
  const sz = Math.sin(z);
  const cz = Math.cos(z);

  const qw = cx * cy * cz + sx * sy * sz;
  const qx = sx * cy * cz - cx * sy * sz;
  const qy = cx * sy * cz + sx * cy * sz;
  const qz = cx * cy * sz - sx * sy * cz;

  return [qx, qy, qz, qw];
}
