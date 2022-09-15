import {
  BufferAttribute,
  BufferGeometry,
  InterleavedBufferAttribute,
  MeshStandardMaterial,
} from "three";

import { AttributeName, ATTRIBUTES } from "../constants";
import { MeshPrimitive } from "../schemaTypes";

export type PrimitiveResult = [
  MeshPrimitive,
  BufferGeometry,
  MeshStandardMaterial
];

export async function loadPrimitive(
  primitiveDef: MeshPrimitive,
  loadAccessor: (
    index: number
  ) => Promise<BufferAttribute | InterleavedBufferAttribute | null>,
  loadMaterial: (index: number) => Promise<MeshStandardMaterial>
): Promise<PrimitiveResult> {
  const geometry = new BufferGeometry();

  // Attributes
  const attributePromises = Object.entries(primitiveDef.attributes).map(
    async ([name, accessorId]) => {
      const threeName = ATTRIBUTES[name as AttributeName] ?? name.toLowerCase();
      if (threeName in geometry.attributes) return;

      const attribute = await loadAccessor(accessorId);

      if (attribute === null) {
        throw new Error(`Attribute ${name} not found`);
      }

      geometry.setAttribute(threeName, attribute);
    }
  );
  await Promise.all(attributePromises);

  // Indices
  if (primitiveDef.indices !== undefined && geometry.index == null) {
    const accessor = await loadAccessor(primitiveDef.indices);

    if (accessor === null) {
      throw new Error(`Indices not found`);
    }

    if (accessor instanceof InterleavedBufferAttribute) {
      throw new Error(`Indices are interleaved`);
    }

    geometry.setIndex(accessor);
  }

  // Material
  const material =
    primitiveDef.material === undefined
      ? new MeshStandardMaterial()
      : await loadMaterial(primitiveDef.material);

  // Occlusion map needs a second set of UVs
  if (
    material.aoMap &&
    geometry.attributes.uv2 === undefined &&
    geometry.attributes.uv !== undefined
  ) {
    geometry.setAttribute("uv2", geometry.attributes.uv);
  }

  // Enable flat shading
  if (geometry.attributes.normal === undefined) {
    material.flatShading = true;
  }

  // Enable vertex colors
  if (geometry.attributes.color !== undefined) {
    material.vertexColors = true;
  }

  // If three.js needs to generate tangents, flip normal map y
  // https://github.com/mrdoob/three.js/issues/11438#issuecomment-507003995
  if (geometry.attributes.tangent === undefined) {
    material.normalScale.y *= -1;
  }

  // Morph targets
  if (primitiveDef.targets !== undefined) {
    const targetPromises = primitiveDef.targets.map((target) => {
      const entryPromises = Object.entries(target).map(
        async ([name, accessorId]) => {
          const accessor = await loadAccessor(accessorId);

          if (accessor === null) {
            throw new Error(`Target ${name} not found`);
          }

          switch (name) {
            case "POSITION":
              if (geometry.morphAttributes.position === undefined) {
                geometry.morphAttributes.position = [];
              }

              geometry.morphAttributes.position.push(accessor);
              break;
            case "NORMAL":
              if (geometry.morphAttributes.normal === undefined) {
                geometry.morphAttributes.normal = [];
              }

              geometry.morphAttributes.normal.push(accessor);
              break;
            case "TANGENT":
              if (geometry.morphAttributes.tangent === undefined) {
                geometry.morphAttributes.tangent = [];
              }

              geometry.morphAttributes.tangent.push(accessor);
              break;
            case "COLOR_0":
              if (geometry.morphAttributes.color === undefined) {
                geometry.morphAttributes.color = [];
              }

              geometry.morphAttributes.color.push(accessor);
              break;
          }
        }
      );

      return Promise.all(entryPromises);
    });

    await Promise.all(targetPromises);

    geometry.morphTargetsRelative = true;
  }

  return [primitiveDef, geometry, material];
}
