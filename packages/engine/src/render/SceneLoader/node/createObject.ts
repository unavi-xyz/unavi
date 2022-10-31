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

import { NodeJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { WEBGL_CONSTANTS } from "../../constants";
import { FromRenderMessage } from "../../types";
import { disposeObject } from "../../utils/disposeObject";
import { defaultMaterial } from "../constants";
import { SceneMap } from "../types";
import { copyTransform } from "../utils/copyTransform";
import { createMeshGeometry } from "./createMeshGeometry";
import { createSkeletons } from "./createSkeletons";
import { removeNodeObject } from "./removeNodeObject";
import { updateMeshMaterial } from "./updateMeshMaterial";
import { updateNode } from "./updateNode";

export function createObject(
  node: NodeJSON,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  const parent = map.objects.get(node.parentId);
  if (!parent) throw new Error("Parent not found");

  const oldObject = map.objects.get(node.id);
  const children = oldObject?.children;

  const mesh = node.meshId ? map.meshes.get(node.meshId) : null;

  // Create object
  switch (mesh?.type) {
    case "Box":
    case "Sphere":
    case "Cylinder": {
      // Get material
      const material = mesh.materialId
        ? map.materials.get(mesh.materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      // Create geometry
      const geometry = createMeshGeometry(mesh, map);

      if (oldObject instanceof Mesh) {
        // Update mesh
        oldObject.geometry.dispose();
        oldObject.geometry = geometry;
        oldObject.material = material;
        copyTransform(oldObject, node);
        parent.add(oldObject);
      } else {
        // Create mesh
        const mesh = new Mesh(geometry, material);

        mesh.castShadow = true;
        mesh.receiveShadow = true;

        // Add to scene
        map.objects.set(node.id, mesh);
        copyTransform(mesh, node);
        parent.add(mesh);
      }
      break;
    }

    case "Primitive": {
      // Remove old object
      if (oldObject) disposeObject(oldObject);

      const isSkin = mesh.skin !== null;

      // Get material
      const primitiveMaterial = node.materialId
        ? map.materials.get(node.materialId)
        : defaultMaterial;
      if (!primitiveMaterial) throw new Error("Material not found");

      // Create geometry
      const primitiveGeometry = createMeshGeometry(mesh, map);

      let primitiveMesh:
        | Mesh
        | SkinnedMesh
        | LineSegments
        | LineLoop
        | Line
        | Points;
      switch (mesh.mode) {
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
          throw new Error(`Unknown primitive mode: ${mesh.mode}`);
      }

      primitiveMesh.castShadow = true;
      primitiveMesh.receiveShadow = true;

      // Set weights
      primitiveMesh.updateMorphTargets();
      primitiveMesh.morphTargetInfluences = [...mesh.weights];

      // Add to scene
      map.objects.set(node.id, primitiveMesh);
      copyTransform(primitiveMesh, node);
      parent.add(primitiveMesh);

      if (isSkin) {
        // Convert all joints to bones
        mesh.skin?.jointIds.forEach((jointId) => {
          const jointNode = map.nodes.get(jointId);
          if (!jointNode) throw new Error(`Node not found: ${jointId}`);

          const jointObject = map.objects.get(jointId);
          if (!jointObject) {
            updateNode(jointNode.id, jointNode, map, visuals, postMessage);
            return;
          }

          removeNodeObject(jointId, map);

          const bone = new Bone();

          // Add to scene
          map.objects.set(jointId, bone);
          copyTransform(bone, jointNode);
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

      map.nodes.forEach((e) => {
        if (
          e.mesh?.type === "Primitive" &&
          e.mesh.skin?.jointIds.includes(node.id)
        ) {
          isJoint = true;
        }
      });

      // Create object
      const object = isJoint ? new Bone() : new Group();

      // Add to scene
      map.objects.set(node.id, object);
      copyTransform(object, node);
      parent.add(object);

      // Update skeletons
      if (isJoint) createSkeletons(map);
    }
  }

  const newObject = map.objects.get(node.id);
  if (!newObject) throw new Error("Object not found");

  // Update mesh material
  updateMeshMaterial(node.id, mesh, map);

  // Set name
  newObject.name = node.name || node.id;

  // Restore children
  if (children && children.length > 0) {
    newObject.add(...children);
  }
}
