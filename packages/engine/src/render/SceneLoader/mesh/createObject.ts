import {
  Bone,
  BoxGeometry,
  BufferGeometry,
  CylinderGeometry,
  Group,
  Line,
  LineLoop,
  LineSegments,
  Mesh,
  Points,
  SkinnedMesh,
  SphereGeometry,
} from "three";

import { PostMessage } from "../../../types";
import { WEBGL_CONSTANTS } from "../../constants";
import { FromRenderMessage } from "../../types";
import { disposeObject } from "../../utils/disposeObject";
import { defaultMaterial } from "../constants";
import { removeNodeObject } from "../node/removeNodeObject";
import { updateNode } from "../node/updateNode";
import { SceneMap } from "../types";
import { copyTransform } from "../utils/copyTransform";
import { createAttribute } from "./createAttribute";
import { createSkeletons } from "./createSkeletons";

export function createObject(
  meshId: string,
  map: SceneMap,
  postMessage: PostMessage<FromRenderMessage>
) {
  function setMorphAttribute(
    geometry: BufferGeometry,
    threeName: string,
    accessorIds: string[]
  ) {
    if (accessorIds.length === 0) return;
    const attributes = accessorIds.map((id) => createAttribute(id, map));
    geometry.morphAttributes[threeName] = attributes;
  }

  function setAttribute(
    geometry: BufferGeometry,
    threeName: string,
    accessorId: string | null
  ) {
    if (accessorId === null) return;

    const attribute = createAttribute(accessorId, map);
    geometry.setAttribute(threeName, attribute);
  }

  const mesh = map.meshes.get(meshId);
  if (!mesh) throw new Error("Mesh not found");

  const oldObject = map.objects.get(mesh.id);

  // Create object
  switch (mesh?.type) {
    case "Box":
    case "Sphere":
    case "Cylinder": {
      // Create geometry
      let geometry: BufferGeometry;

      switch (mesh.type) {
        case "Box": {
          geometry = new BoxGeometry(mesh.width, mesh.height, mesh.depth);
          break;
        }

        case "Sphere": {
          geometry = new SphereGeometry(
            mesh.radius,
            mesh.widthSegments,
            mesh.heightSegments
          );
          break;
        }

        case "Cylinder": {
          geometry = new CylinderGeometry(
            mesh.radius,
            mesh.radius,
            mesh.height,
            mesh.radialSegments
          );
          break;
        }
      }

      // Get material
      const material = mesh.materialId
        ? map.materials.get(mesh.materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      // Create mesh
      const threeMesh = new Mesh(geometry, material);
      threeMesh.castShadow = true;
      threeMesh.receiveShadow = true;

      map.objects.set(mesh.id, threeMesh);
      oldObject?.parent?.add(threeMesh);
      break;
    }

    case "Primitives": {
      const primitivesGroup = new Group();
      map.objects.set(mesh.id, primitivesGroup);
      oldObject?.parent?.add(primitivesGroup);

      mesh.primitives.map((primitive) => {
        // Create geometry
        const geometry = new BufferGeometry();
        geometry.morphTargetsRelative = true;

        // Set indices
        if (primitive.indicesId) {
          const attribute = createAttribute(primitive.indicesId, map);
          geometry.setIndex(attribute);
        }

        // Set attributes
        setAttribute(geometry, "position", primitive.POSITION);
        setAttribute(geometry, "normal", primitive.NORMAL);
        setAttribute(geometry, "uv", primitive.TEXCOORD_0);
        setAttribute(geometry, "uv2", primitive.TEXCOORD_1);
        setAttribute(geometry, "color", primitive.COLOR_0);
        setAttribute(geometry, "skinIndex", primitive.JOINTS_0);
        setAttribute(geometry, "skinWeight", primitive.WEIGHTS_0);

        // Set morph targets
        setMorphAttribute(geometry, "position", primitive.morphPositionIds);
        setMorphAttribute(geometry, "normal", primitive.morphNormalIds);
        setMorphAttribute(geometry, "tangent", primitive.morphTangentIds);

        // Get material
        const primitiveMaterial = primitive.materialId
          ? map.materials.get(primitive.materialId)
          : defaultMaterial;
        if (!primitiveMaterial) throw new Error("Material not found");

        let primitiveMesh:
          | Mesh
          | SkinnedMesh
          | LineSegments
          | LineLoop
          | Line
          | Points;
        switch (primitive.mode) {
          case WEBGL_CONSTANTS.TRIANGLES:
          case WEBGL_CONSTANTS.TRIANGLE_STRIP:
          case WEBGL_CONSTANTS.TRIANGLE_FAN: {
            primitiveMesh = primitive.skin
              ? new SkinnedMesh(geometry, primitiveMaterial)
              : new Mesh(geometry, primitiveMaterial);

            if (primitiveMesh instanceof SkinnedMesh) {
              const normalized =
                primitiveMesh.geometry.attributes.skinWeight.normalized;
              if (!normalized) primitiveMesh.normalizeSkinWeights();
            }
            break;
          }

          case WEBGL_CONSTANTS.LINES: {
            primitiveMesh = new LineSegments(geometry, primitiveMaterial);
            break;
          }

          case WEBGL_CONSTANTS.LINE_STRIP: {
            primitiveMesh = new Line(geometry, primitiveMaterial);
            break;
          }

          case WEBGL_CONSTANTS.LINE_LOOP: {
            primitiveMesh = new LineLoop(geometry, primitiveMaterial);
            break;
          }

          case WEBGL_CONSTANTS.POINTS: {
            primitiveMesh = new Points(geometry, primitiveMaterial);
            break;
          }

          default: {
            throw new Error(`Unknown primitive mode: ${primitive.mode}`);
          }
        }

        primitiveMesh.castShadow = true;
        primitiveMesh.receiveShadow = true;

        // Set weights
        primitiveMesh.updateMorphTargets();
        primitiveMesh.morphTargetInfluences = [...primitive.weights];

        // Add to group
        primitivesGroup.add(primitiveMesh);
        map.objects.set(primitive.id, primitiveMesh);

        // Convert all joints to bones
        primitive.skin?.jointIds.forEach((jointId) => {
          const jointNode = map.nodes.get(jointId);
          if (!jointNode) return;

          const jointObject = map.objects.get(jointId);
          if (!jointObject) {
            updateNode(jointNode.id, jointNode, map, postMessage);
            return;
          }

          removeNodeObject(jointId, map);

          const bone = new Bone();
          map.objects.set(jointId, bone);
          copyTransform(bone, jointNode);
        });
      });

      // Create skeletons
      createSkeletons(map);
      break;
    }

    default: {
      // Create empty object
      const object = new Group();
      oldObject?.parent?.add(object);
      map.objects.set(mesh.id, object);
    }
  }

  // Remove old object
  if (oldObject) disposeObject(oldObject);
}
