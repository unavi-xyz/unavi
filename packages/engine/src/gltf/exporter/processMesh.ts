import {
  BufferAttribute,
  BufferGeometry,
  InterleavedBuffer,
  InterleavedBufferAttribute,
  Line,
  LineLoop,
  LineSegments,
  Material,
  Mesh,
  Points,
} from "three";

import { ATTRIBUTES, THREE_ATTTRIBUTES, WEBGL_CONSTANTS } from "../constants";
import { GLTF, Mesh as MeshDef, MeshPrimitive } from "../schemaTypes";
import { createNormalizedNormalAttribute } from "./createNormalizedNormalAttribute";
import { isNormalizedNormalAttribute } from "./isNormalizedNormalAttribute";
import { ProcessAccessorOptions } from "./processAccessor";

export function processMesh(
  mesh: Mesh | Line | Points,
  json: GLTF,
  processAccessor: (
    attribute: BufferAttribute | InterleavedBufferAttribute,
    options?: ProcessAccessorOptions
  ) => number | null,
  processMaterial: (material: Material) => number
) {
  const geometry = mesh.geometry;
  if (!(geometry instanceof BufferGeometry)) throw new Error("Geometry must be a BufferGeometry.");

  const meshDef: MeshDef = { primitives: [] };
  if (mesh.name) meshDef.name = mesh.name;

  // Set mode
  let mode: number;

  if (mesh instanceof LineSegments) {
    mode = WEBGL_CONSTANTS.LINES;
  } else if (mesh instanceof LineLoop) {
    mode = WEBGL_CONSTANTS.LINE_LOOP;
  } else if (mesh instanceof Line) {
    mode = WEBGL_CONSTANTS.LINE_STRIP;
  } else if (mesh instanceof Points) {
    mode = WEBGL_CONSTANTS.POINTS;
  } else {
    mode = WEBGL_CONSTANTS.TRIANGLES;
  }

  // Normalize the normal attribute, if not already normalized
  const normal = geometry.getAttribute(ATTRIBUTES.NORMAL);
  if (normal instanceof BufferAttribute && !isNormalizedNormalAttribute(normal)) {
    const normalized = createNormalizedNormalAttribute(normal);
    geometry.setAttribute(ATTRIBUTES.NORMAL, normalized);
  }

  // Attributes
  const attributes: { [key: string]: number } = {};
  const attributeNames = Object.keys(geometry.attributes);
  let interleavedArray = new Float32Array(0);
  let interleavedStride: number = 0;

  const attributeTypes = attributeNames.map((name) => {
    const attribute = geometry.getAttribute(name);
    const componentType = attribute.array.constructor.name;
    return componentType;
  });

  const interleave =
    attributeTypes.every((type) => type === "Float32Array") && attributeTypes.length > 1;

  if (interleave) {
    // Set stride and length
    let length = 0;

    attributeNames.forEach((name) => {
      const attribute = geometry.getAttribute(name);
      interleavedStride += attribute.itemSize;
      length += attribute.count * attribute.itemSize;
    });

    // Build interleaved array
    interleavedArray = new Float32Array(length);
    let interleaveOffset = 0;

    const count = length / interleavedStride;
    for (let i = 0; i < count; i++) {
      attributeNames.forEach((name) => {
        const attribute = geometry.getAttribute(name);
        const itemSize = attribute.itemSize;

        if (attribute instanceof BufferAttribute) {
          for (let j = 0; j < itemSize; j++) {
            const value = attribute.array[i * itemSize + j];
            interleavedArray[interleaveOffset + j] = value;
          }
        } else {
          const stride = attribute.data.stride;
          const offset = attribute.offset;

          for (let j = 0; j < itemSize; j++) {
            const value = attribute.array[offset + i * stride + j];
            interleavedArray[interleaveOffset + j] = value;
          }
        }

        interleaveOffset += itemSize;
      });
    }
  }

  const interleavedBuffer = new InterleavedBuffer(interleavedArray, interleavedStride);

  let interleaveOffset = 0;

  attributeNames.forEach((name) => {
    // Ignore morph targets, which are exported later
    if (name.slice(0, 5) === "morph") return;

    let attribute = geometry.attributes[name];
    const originalName = attribute.name;
    // @ts-ignore
    const attributeName = THREE_ATTTRIBUTES[name] ?? `_${name.toUpperCase()}`;

    // JOINTS_0 must be UNSIGNED_BYTE or UNSIGNED_SHORT
    if (
      attributeName === "JOINTS_0" &&
      !(attribute.array instanceof Uint16Array) &&
      !(attribute.array instanceof Uint8Array)
    ) {
      attribute = new BufferAttribute(
        new Uint16Array(attribute.array),
        attribute.itemSize,
        attribute.normalized
      );
    }

    if (interleave) {
      // Convert to interleaved attribute
      attribute = new InterleavedBufferAttribute(
        interleavedBuffer,
        attribute.itemSize,
        interleaveOffset,
        attribute.normalized
      );
    }

    interleaveOffset += attribute.itemSize;
    if (originalName) attribute.name = originalName;

    const accessorIndex = processAccessor(attribute, { geometry, interleavedBuffer });
    if (accessorIndex !== null) attributes[attributeName] = accessorIndex;
  });

  // Set the normal back to the original value
  if (normal !== undefined) geometry.setAttribute(ATTRIBUTES.NORMAL, normal);

  // If no exportable attributes, skip the mesh
  if (Object.keys(attributes).length === 0) return null;

  // Morph targets
  const targets: { [k: string]: number }[] = [];
  const weights: number[] = [];

  if (mesh.morphTargetInfluences) {
    mesh.morphTargetInfluences.forEach((_, i) => {
      const target: { [key: string]: number } = {};

      for (const attributeName in geometry.morphAttributes) {
        // glTF 2.0 morph supports POSITION/NORMAL/TANGENT.
        // Three.js doesn't support TANGENT yet.
        if (attributeName !== ATTRIBUTES.POSITION && attributeName !== ATTRIBUTES.NORMAL) continue;

        const attribute = geometry.morphAttributes[attributeName][i];
        const gltfAttributeName = attributeName.toUpperCase();

        // Three.js morph attribute has absolute values while the one of glTF has relative values.
        const baseAttribute = geometry.attributes[attributeName];
        const relativeAttribute = attribute.clone();

        if (!geometry.morphTargetsRelative) {
          for (let i = 0; i < relativeAttribute.count; i++) {
            relativeAttribute.setXYZ(
              i,
              attribute.getX(i) - baseAttribute.getX(i),
              attribute.getY(i) - baseAttribute.getY(i),
              attribute.getZ(i) - baseAttribute.getZ(i)
            );
          }
        }

        const accessorIndex = processAccessor(relativeAttribute, { geometry });

        if (accessorIndex !== null) {
          target[gltfAttributeName] = accessorIndex;
        }
      }

      targets.push(target);
      if (mesh.morphTargetInfluences) weights.push(mesh.morphTargetInfluences[i]);
    });

    meshDef.weights = weights;
  }

  // Primitives
  if (Array.isArray(mesh.material) && geometry.groups.length === 0) return null;

  const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
  const groups = Array.isArray(mesh.material)
    ? geometry.groups
    : [{ materialIndex: 0, start: undefined, count: undefined }];

  const primitives = groups.map((group) => {
    const primitive: MeshPrimitive = {
      mode,
      attributes,
    };

    if (targets.length > 0) primitive.targets = targets;

    if (geometry.index) {
      const accessor = processAccessor(geometry.index, {
        geometry,
        start: group.start,
        count: group.count,
      });
      if (accessor !== null) primitive.indices = accessor;
    }

    if (group.materialIndex !== undefined) {
      const material = materials[group.materialIndex];
      const index = processMaterial(material);
      primitive.material = index;
    }

    return primitive;
  });

  meshDef.primitives = primitives;

  if (!json.meshes) json.meshes = [];
  const index = json.meshes.push(meshDef) - 1;
  return index;
}
