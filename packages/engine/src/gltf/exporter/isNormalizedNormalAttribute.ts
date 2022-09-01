import { BufferAttribute, Vector3 } from "three";

export function isNormalizedNormalAttribute(normal: BufferAttribute): boolean {
  const vector = new Vector3();

  for (let i = 0, il = normal.count; i < il; i++) {
    // 0.0005 is from glTF-validator
    if (Math.abs(vector.fromBufferAttribute(normal, i).length() - 1.0) > 0.0005)
      return false;
  }

  return true;
}
