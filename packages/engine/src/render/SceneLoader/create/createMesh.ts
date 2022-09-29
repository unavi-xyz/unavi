import {
  Bone,
  Group,
  Line,
  LineLoop,
  LineSegments,
  Mesh,
  Points,
  SkinnedMesh,
} from "three";

import { MeshJSON } from "../../../scene";
import { WEBGL_CONSTANTS } from "../../constants";
import { RenderScene } from "../../RenderScene";
import { defaultMaterial } from "../constants";
import { removeEntityObject } from "../remove/removeEntityObject";
import { SceneMap } from "../types";
import { copyTransform } from "../utils/copyTransform";
import { createEntity } from "./createEntity";
import { createMeshGeometry } from "./createMeshGeometry";
import { createSkeletons } from "./createSkeletons";

export function createMesh(
  entityId: string,
  json: MeshJSON | undefined,
  map: SceneMap,
  scene: RenderScene,
  visuals: Group
) {
  const entity = scene.entities[entityId];
  const parent = map.objects.get(entity.parentId);
  if (!parent) throw new Error("Parent not found");

  // Create object
  switch (json?.type) {
    case "Box":
    case "Sphere":
    case "Cylinder":
      // Get material
      const material = entity.materialId
        ? map.materials.get(entity.materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      // Create geometry
      const geometry = createMeshGeometry(json, map, scene);

      // Create mesh
      const mesh = new Mesh(geometry, material);

      // Add to scene
      map.objects.set(entity.id, mesh);
      copyTransform(mesh, entity);
      parent.add(mesh);
      break;
    case "Primitive":
      const isSkin = json.skin !== null;

      // Get material
      const primitiveMaterial = entity.materialId
        ? map.materials.get(entity.materialId)
        : defaultMaterial;
      if (!primitiveMaterial) throw new Error("Material not found");

      // Create geometry
      const primitiveGeometry = createMeshGeometry(json, map, scene);

      let primitiveMesh:
        | Mesh
        | SkinnedMesh
        | LineSegments
        | LineLoop
        | Line
        | Points;
      switch (json.mode) {
        case WEBGL_CONSTANTS.TRIANGLES:
        case WEBGL_CONSTANTS.TRIANGLE_STRIP:
        case WEBGL_CONSTANTS.TRIANGLE_FAN:
          primitiveMesh = isSkin
            ? new SkinnedMesh(primitiveGeometry, primitiveMaterial)
            : new Mesh(primitiveGeometry, primitiveMaterial);

          if (primitiveMesh instanceof SkinnedMesh) {
            const normalized =
              primitiveMesh.geometry.attributes.skinWeight.normalized;
            if (!normalized) primitiveMesh.normalizeSkinWeights();
          }

          break;
        case WEBGL_CONSTANTS.LINES:
          primitiveMesh = new LineSegments(
            primitiveGeometry,
            primitiveMaterial
          );
          break;
        case WEBGL_CONSTANTS.LINE_STRIP:
          primitiveMesh = new Line(primitiveGeometry, primitiveMaterial);
          break;
        case WEBGL_CONSTANTS.LINE_LOOP:
          primitiveMesh = new LineLoop(primitiveGeometry, primitiveMaterial);
          break;
        case WEBGL_CONSTANTS.POINTS:
          primitiveMesh = new Points(primitiveGeometry, primitiveMaterial);
          break;
        default:
          throw new Error(`Unknown primitive mode: ${json.mode}`);
      }

      // Occlusion map needs a second set of UVs
      if (
        primitiveMesh.material.aoMap &&
        primitiveMesh.geometry.attributes.uv &&
        !primitiveMesh.geometry.attributes.uv2
      ) {
        primitiveMesh.geometry.setAttribute(
          "uv2",
          primitiveMesh.geometry.attributes.uv
        );
      }

      // Enable flat shading if no normal attribute
      if (!primitiveMesh.geometry.attributes.normal)
        primitiveMesh.material.flatShading = true;

      // Enable vertex colors if color attribute
      if (primitiveMesh.geometry.attributes.color)
        primitiveMesh.material.vertexColors = true;

      // If three.js needs to generate tangents, flip normal map y
      if (!primitiveMesh.geometry.attributes.tangent)
        primitiveMesh.material.normalScale.y *= -1;

      // Set weights
      primitiveMesh.updateMorphTargets();
      primitiveMesh.morphTargetInfluences = [...json.weights];

      // Add to scene
      map.objects.set(entity.id, primitiveMesh);
      copyTransform(primitiveMesh, entity);
      parent.add(primitiveMesh);

      if (isSkin) {
        // Convert all joints to bones
        json.skin?.jointIds.forEach((jointId) => {
          const jointObject = map.objects.get(jointId);
          if (jointObject instanceof Bone) return;

          if (!jointObject) {
            const jointEntity = scene.entities[jointId];
            createEntity(jointEntity, map, scene, visuals);
            return;
          }

          removeEntityObject(jointId, map);

          const bone = new Bone();

          // Add to scene
          const joint = scene.entities[jointId];
          map.objects.set(jointId, bone);
          copyTransform(bone, joint);
          parent.add(bone);
        });
      }

      // Create skeletons
      createSkeletons(scene, map);
      break;
    default:
      // Check if joint
      let isJoint = false;

      Object.values(scene.entities).forEach((e) => {
        if (
          e.mesh?.type === "Primitive" &&
          e.mesh.skin?.jointIds.includes(entity.id)
        ) {
          isJoint = true;
        }
      });

      // Create object
      const object = isJoint ? new Bone() : new Group();

      // Add to scene
      map.objects.set(entity.id, object);
      copyTransform(object, entity);
      parent.add(object);

      // Update skeletons
      if (isJoint) createSkeletons(scene, map);
  }
}
