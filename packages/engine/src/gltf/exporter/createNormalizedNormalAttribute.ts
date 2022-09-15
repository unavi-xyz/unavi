import { BufferAttribute, Vector3 } from "three";

export function createNormalizedNormalAttribute(normal: BufferAttribute) {
  const attribute = normal.clone();
  const vector = new Vector3();

  for (let i = 0, il = attribute.count; i < il; i++) {
    vector.fromBufferAttribute(attribute, i);

    if (vector.x === 0 && vector.y === 0 && vector.z === 0) {
      // if values can't be normalized set (1, 0, 0)
      vector.setX(1.0);
    } else {
      vector.normalize();
    }

    attribute.setXYZ(i, vector.x, vector.y, vector.z);
  }

  return attribute;
}
