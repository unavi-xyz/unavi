import {
  BufferAttribute,
  InterleavedBuffer,
  InterleavedBufferAttribute,
} from "three";

import {
  ComponentType,
  TypeSize,
  WEBGL_COMPONENT_TYPES,
  WEBGL_TYPE_SIZES,
} from "../constants";
import { BufferViewResult } from "../loader/loadBufferView";
import { BufferView, GLTF } from "../schemaTypes";

export type AccessorResult =
  | BufferAttribute
  | InterleavedBufferAttribute
  | null;

export async function loadAccessor(
  index: number,
  json: GLTF,
  bufferViews: BufferViewResult[],
  interleavedBuffers: Map<string, InterleavedBuffer>
): Promise<AccessorResult> {
  if (json.accessors === undefined) {
    throw new Error("No accessors found");
  }

  const accessorDef = json.accessors[index];

  if (
    accessorDef.bufferView === undefined &&
    accessorDef.sparse === undefined
  ) {
    // Ignore empty accessors, which may be used to declare runtime
    // information about attributes coming from another source (e.g. Draco
    // compression extension).
    return null;
  }

  // Load buffer views
  const views: ({
    bufferViewDef: BufferView;
    bufferView: ArrayBuffer;
  } | null)[] = [];

  if (accessorDef.bufferView !== undefined) {
    const bufferView = await bufferViews[accessorDef.bufferView];
    views.push(bufferView);
  } else {
    views.push(null);
  }

  if (accessorDef.sparse !== undefined) {
    const indices = await bufferViews[accessorDef.sparse.indices.bufferView];
    const values = await bufferViews[accessorDef.sparse.values.bufferView];

    views.push(indices, values);
  }

  if (accessorDef.componentType in WEBGL_COMPONENT_TYPES === false) {
    throw new Error(`Invalid component type: ${accessorDef.componentType}`);
  }

  const componentType = accessorDef.componentType as ComponentType;
  const TypedArray = WEBGL_COMPONENT_TYPES[componentType];

  if (accessorDef.type in WEBGL_TYPE_SIZES === false) {
    throw new Error(`Invalid type: ${accessorDef.type}`);
  }

  const { bufferViewDef, bufferView } = views[0]
    ? views[0]
    : { bufferViewDef: null, bufferView: null };

  const type = accessorDef.type as TypeSize;
  const itemSize = WEBGL_TYPE_SIZES[type];

  const elementBytes = TypedArray.BYTES_PER_ELEMENT;
  const itemBytes = elementBytes * itemSize;
  const byteStride = bufferViewDef?.byteStride;
  const byteOffset = accessorDef.byteOffset ?? 0;
  const normalized = accessorDef.normalized;

  // Create buffer attribute
  let bufferAttribute: BufferAttribute | InterleavedBufferAttribute;
  const isInterleaved = byteStride && byteStride !== itemBytes && bufferView;

  if (isInterleaved) {
    const slice = Math.floor(byteOffset / byteStride);
    const cacheKey = `${accessorDef.bufferView}-${accessorDef.componentType}-${slice}-${accessorDef.count}`;
    let interleavedBuffer = interleavedBuffers.get(cacheKey);

    if (interleavedBuffer === undefined) {
      const array = new TypedArray(
        bufferView,
        slice * byteStride,
        (accessorDef.count * byteStride) / elementBytes
      );
      // console.log("🙆‍♀️", array);
      interleavedBuffer = new InterleavedBuffer(
        array,
        byteStride / elementBytes
      );

      interleavedBuffers.set(cacheKey, interleavedBuffer);
    }

    if (interleavedBuffer === undefined) {
      throw new Error("Failed to create interleaved buffer");
    }

    bufferAttribute = new InterleavedBufferAttribute(
      interleavedBuffer,
      itemSize,
      (byteOffset % byteStride) / elementBytes,
      normalized
    );
  } else {
    let array: ArrayLike<number>;

    if (bufferView === null) {
      array = new TypedArray(accessorDef.count * itemSize);
    } else {
      array = new TypedArray(
        bufferView,
        byteOffset,
        accessorDef.count * itemSize
      );
    }

    bufferAttribute = new BufferAttribute(array, itemSize, normalized);
  }

  // Sparse
  if (
    accessorDef.sparse !== undefined &&
    views[1] !== null &&
    views[2] !== null
  ) {
    const itemSizeIndices = WEBGL_TYPE_SIZES.SCALAR;
    const indicesComponentType = accessorDef.sparse.indices
      .componentType as ComponentType;

    if (indicesComponentType in WEBGL_COMPONENT_TYPES === false) {
      throw new Error(`Invalid component type: ${accessorDef.componentType}`);
    }

    const TypedArrayIndices = WEBGL_COMPONENT_TYPES[indicesComponentType];

    const byteOffsetIndices = accessorDef.sparse.indices.byteOffset ?? 0;
    const byteOffsetValues = accessorDef.sparse.values.byteOffset ?? 0;

    const sparseIndices = new TypedArrayIndices(
      views[1].bufferView,
      byteOffsetIndices,
      accessorDef.sparse.count * itemSizeIndices
    );
    const sparseValues = new TypedArray(
      views[2].bufferView,
      byteOffsetValues,
      accessorDef.sparse.count * itemSize
    );

    if (bufferView !== null) {
      // Avoid modifying the original ArrayBuffer, if the bufferView wasn't initialized with zeroes.
      bufferAttribute = new BufferAttribute(
        new TypedArray(bufferAttribute.array),
        itemSize,
        normalized
      );
    }

    sparseIndices.forEach((index, i) => {
      bufferAttribute.setX(index, sparseValues[i * itemSize]);
      if (itemSize >= 2)
        bufferAttribute.setY(index, sparseValues[i * itemSize + 1]);
      if (itemSize >= 3)
        bufferAttribute.setZ(index, sparseValues[i * itemSize + 2]);
      if (itemSize >= 4)
        bufferAttribute.setW(index, sparseValues[i * itemSize + 3]);
      if (itemSize >= 5)
        throw new Error("Unsupported itemSize in sparse BufferAttribute.");
    });
  }

  bufferAttribute.name = accessorDef.name ?? `bufferAttribute_${index}`;
  return bufferAttribute;
}
