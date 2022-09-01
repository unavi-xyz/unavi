import {
  Group,
  Line,
  LineLoop,
  LineSegments,
  Mesh,
  Object3D,
  Points,
  SkinnedMesh,
} from "three";

import { WEBGL_CONSTANTS } from "../constants";
import { GLTF, MeshPrimitive } from "../schemaTypes";
import { PrimitiveResult } from "./loadPrimitive";

export async function loadMesh(
  index: number,
  json: GLTF,
  skinnedMeshIndexes: Set<number>,
  loadPrimitive: (primitiveDef: MeshPrimitive) => Promise<PrimitiveResult>
): Promise<Object3D> {
  if (json.meshes === undefined) {
    throw new Error("No meshes found");
  }

  const meshDef = json.meshes[index];

  // Load primtives
  const primitivePromises = meshDef.primitives.map((primitive) =>
    loadPrimitive(primitive)
  );
  const primitives = await Promise.all(primitivePromises);

  // Create meshes
  const meshes = primitives.map(([primitive, geometry, material], i) => {
    let mesh: Mesh | Line | Points;

    switch (primitive.mode) {
      case WEBGL_CONSTANTS.TRIANGLES:
      case WEBGL_CONSTANTS.TRIANGLE_STRIP:
      case WEBGL_CONSTANTS.TRIANGLE_FAN:
      case undefined:
        const isSkinnedMesh = skinnedMeshIndexes.has(index);
        mesh = isSkinnedMesh
          ? new SkinnedMesh(geometry, material)
          : new Mesh(geometry, material);

        if (
          mesh instanceof SkinnedMesh &&
          !mesh.geometry.attributes.skinWeight.normalized
        ) {
          mesh.normalizeSkinWeights();
        }

        break;
      case WEBGL_CONSTANTS.LINES:
        mesh = new LineSegments(geometry, material);
        break;
      case WEBGL_CONSTANTS.LINE_STRIP:
        mesh = new Line(geometry, material);
        break;
      case WEBGL_CONSTANTS.LINE_LOOP:
        mesh = new LineLoop(geometry, material);
        break;
      case WEBGL_CONSTANTS.POINTS:
        mesh = new Points(geometry, material);
        break;
      default:
        throw new Error(`Unknown primitive mode ${primitive.mode}`);
    }

    mesh.name = meshDef.name ?? `mesh_${index}_${i}`;
    return mesh;
  });

  // Set morph targets
  meshes.forEach((mesh) => {
    if (Object.keys(mesh.geometry.morphAttributes).length > 0) {
      mesh.updateMorphTargets();

      if (meshDef.weights !== undefined) {
        mesh.morphTargetInfluences = [...meshDef.weights];
      }
    }
  });

  const mesh = meshes.length === 1 ? meshes[0] : new Group().add(...meshes);
  return mesh;
}
