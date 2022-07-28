import {
  BufferAttribute,
  BufferGeometry,
  InterleavedBufferAttribute,
  Line,
  LineLoop,
  LineSegments,
  Material,
  Mesh,
  Points,
} from "three";

import { ATTRIBUTES, THREE_ATTTRIBUTES, ThreeAttributeName, WEBGL_CONSTANTS } from "../constants";
import { GLTF, Mesh as MeshDef, MeshPrimitive } from "../schemaTypes";
import { createNormalizedNormalAttribute } from "./createNormalizedNormalAttribute";
import { isNormalizedNormalAttribute } from "./isNormalizedNormalAttribute";

export function processMesh(
  mesh: Mesh | Line | Points,
  json: GLTF,
  processAccessor: (
    attribute: BufferAttribute | InterleavedBufferAttribute,
    geometry: BufferGeometry,
    count?: number,
    start?: number
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

  for (const name in geometry.attributes) {
    // Ignore morph targets, which are exported later
    if (name.slice(0, 5) === "morph") continue;

    const attribute = geometry.attributes[name];
    const attributeName = THREE_ATTTRIBUTES[name as ThreeAttributeName] ?? `_${name.toUpperCase()}`;

    // JOINTS_0 must be UNSIGNED_BYTE or UNSIGNED_SHORT
    let modifiedAttribute = attribute;
    const array = attribute.array;
    if (
      attributeName === "JOINTS_0" &&
      !(array instanceof Uint16Array) &&
      !(array instanceof Uint8Array)
    ) {
      modifiedAttribute = new BufferAttribute(
        new Uint16Array(array),
        attribute.itemSize,
        attribute.normalized
      );
    }

    const accessorIndex = processAccessor(modifiedAttribute, geometry);

    if (accessorIndex !== null) {
      attributes[attributeName] = accessorIndex;
    }
  }

  // Set the normal back to the original value
  if (normal !== undefined) geometry.setAttribute(ATTRIBUTES.NORMAL, normal);

  // If no exportable attributes, skip the mesh
  if (Object.keys(attributes).length === 0) return null;

  // Morph targets
  const targets: {
    [k: string]: number;
  }[] = [];
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

        const accessorIndex = processAccessor(relativeAttribute, geometry);

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
      const accessor = processAccessor(geometry.index, geometry, group.start, group.count);
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
