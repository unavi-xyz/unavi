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

import { EntityJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { WEBGL_CONSTANTS } from "../../constants";
import { FromRenderMessage } from "../../types";
import { disposeObject } from "../../utils/disposeObject";
import { defaultMaterial } from "../constants";
import { SceneMap } from "../types";
import { copyTransform } from "../utils/copyTransform";
import { createMeshGeometry } from "./createMeshGeometry";
import { createSkeletons } from "./createSkeletons";
import { removeEntityObject } from "./removeEntityObject";
import { updateEntity } from "./updateEntity";
import { updateMeshMaterial } from "./updateMeshMaterial";

export function createObject(
  entity: EntityJSON,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  const parent = map.objects.get(entity.parentId);
  if (!parent) throw new Error("Parent not found");

  const oldObject = map.objects.get(entity.id);
  const children = oldObject?.children;

  // Create object
  switch (entity.mesh?.type) {
    case "Box":
    case "Sphere":
    case "Cylinder": {
      // Get material
      const material = entity.materialId
        ? map.materials.get(entity.materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      // Create geometry
      const geometry = createMeshGeometry(entity.mesh, map);

      if (oldObject instanceof Mesh) {
        // Update mesh
        oldObject.geometry.dispose();
        oldObject.geometry = geometry;
        oldObject.material = material;
        copyTransform(oldObject, entity);
        parent.add(oldObject);
      } else {
        // Create mesh
        const mesh = new Mesh(geometry, material);

        mesh.castShadow = true;
        mesh.receiveShadow = true;

        // Add to scene
        map.objects.set(entity.id, mesh);
        copyTransform(mesh, entity);
        parent.add(mesh);
      }
      break;
    }

    case "Primitive": {
      // Remove old object
      if (oldObject) disposeObject(oldObject);

      const isSkin = entity.mesh.skin !== null;

      // Get material
      const primitiveMaterial = entity.materialId
        ? map.materials.get(entity.materialId)
        : defaultMaterial;
      if (!primitiveMaterial) throw new Error("Material not found");

      // Create geometry
      const primitiveGeometry = createMeshGeometry(entity.mesh, map);

      let primitiveMesh:
        | Mesh
        | SkinnedMesh
        | LineSegments
        | LineLoop
        | Line
        | Points;
      switch (entity.mesh.mode) {
        case WEBGL_CONSTANTS.TRIANGLES:
        case WEBGL_CONSTANTS.TRIANGLE_STRIP:
        case WEBGL_CONSTANTS.TRIANGLE_FAN: {
          primitiveMesh = isSkin
            ? new SkinnedMesh(primitiveGeometry, primitiveMaterial)
            : new Mesh(primitiveGeometry, primitiveMaterial);

          if (primitiveMesh instanceof SkinnedMesh) {
            const normalized =
              primitiveMesh.geometry.attributes.skinWeight.normalized;
            if (!normalized) primitiveMesh.normalizeSkinWeights();
          }
          break;
        }

        case WEBGL_CONSTANTS.LINES: {
          primitiveMesh = new LineSegments(
            primitiveGeometry,
            primitiveMaterial
          );
          break;
        }

        case WEBGL_CONSTANTS.LINE_STRIP: {
          primitiveMesh = new Line(primitiveGeometry, primitiveMaterial);
          break;
        }

        case WEBGL_CONSTANTS.LINE_LOOP: {
          primitiveMesh = new LineLoop(primitiveGeometry, primitiveMaterial);
          break;
        }

        case WEBGL_CONSTANTS.POINTS: {
          primitiveMesh = new Points(primitiveGeometry, primitiveMaterial);
          break;
        }

        default:
          throw new Error(`Unknown primitive mode: ${entity.mesh.mode}`);
      }

      primitiveMesh.castShadow = true;
      primitiveMesh.receiveShadow = true;

      // Set weights
      primitiveMesh.updateMorphTargets();
      primitiveMesh.morphTargetInfluences = [...entity.mesh.weights];

      // Add to scene
      map.objects.set(entity.id, primitiveMesh);
      copyTransform(primitiveMesh, entity);
      parent.add(primitiveMesh);

      if (isSkin) {
        // Convert all joints to bones
        entity.mesh.skin?.jointIds.forEach((jointId) => {
          const jointEntity = map.entities.get(jointId);
          if (!jointEntity) throw new Error(`Entity not found: ${jointId}`);

          const jointObject = map.objects.get(jointId);
          if (!jointObject) {
            updateEntity(
              jointEntity.id,
              jointEntity,
              map,
              visuals,
              postMessage
            );
            return;
          }

          removeEntityObject(jointId, map);

          const bone = new Bone();

          // Add to scene
          map.objects.set(jointId, bone);
          copyTransform(bone, jointEntity);
          parent.add(bone);
        });
      }

      // Create skeletons
      createSkeletons(map);
      break;
    }

    default: {
      // Remove old object
      if (oldObject) disposeObject(oldObject);

      // Check if joint
      let isJoint = false;

      map.entities.forEach((e) => {
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
      if (isJoint) createSkeletons(map);
    }
  }

  const newObject = map.objects.get(entity.id);
  if (!newObject) throw new Error("Object not found");

  // Update mesh material
  updateMeshMaterial(entity.id, entity.mesh, map);

  // Set name
  newObject.name = entity.name || entity.id;

  // Restore children
  if (children && children.length > 0) {
    newObject.add(...children);
  }
}
