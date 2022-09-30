import { Quad, Triplet } from "@wired-labs/engine";

export function eulerToQuaternion(euler: Triplet): Quad {
  const c1 = Math.cos(euler[0] / 2);
  const c2 = Math.cos(euler[1] / 2);
  const c3 = Math.cos(euler[2] / 2);
  const s1 = Math.sin(euler[0] / 2);
  const s2 = Math.sin(euler[1] / 2);
  const s3 = Math.sin(euler[2] / 2);

  return [
    c1 * c2 * c3 - s1 * s2 * s3,
    s1 * s2 * c3 + c1 * c2 * s3,
    s1 * c2 * c3 + c1 * s2 * s3,
    c1 * s2 * c3 - s1 * c2 * s3,
  ];
}
