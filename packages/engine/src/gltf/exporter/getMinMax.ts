import { BufferAttribute, InterleavedBufferAttribute, Vector3 } from "three";

// Get the min and max vectors from the given attribute
export function getMinMax(
  attribute: BufferAttribute | InterleavedBufferAttribute,
  start: number,
  count: number
) {
  const output = {
    min: new Array(attribute.itemSize).fill(Number.POSITIVE_INFINITY),
    max: new Array(attribute.itemSize).fill(Number.NEGATIVE_INFINITY),
  };

  for (let i = start; i < start + count; i++) {
    for (let j = 0; j < attribute.itemSize; j++) {
      let value: number | undefined = undefined;

      if (attribute.itemSize > 4) {
        // no support for interleaved data for itemSize > 4
        value = attribute.array[i * attribute.itemSize + j];
      } else {
        switch (j) {
          case 0:
            value = attribute.getX(i);
            break;
          case 1:
            value = attribute.getY(i);
            break;
          case 2:
            value = attribute.getZ(i);
            break;
          case 3:
            value = attribute.getW(i);
            break;
        }
      }

      if (value === undefined) continue;

      output.min[j] = Math.min(output.min[j], value);
      output.max[j] = Math.max(output.max[j], value);
    }
  }

  return output;
}
