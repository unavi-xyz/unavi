import {
  BoxGeometry,
  BufferGeometry,
  CylinderGeometry,
  Group,
  Mesh,
  MeshBasicMaterial,
  Quaternion,
  SphereGeometry,
  Vector3,
} from "three";
import { mergeBufferGeometries } from "three/examples/jsm/utils/BufferGeometryUtils";

import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { SceneMap } from "../types";
import { removeColliderVisual } from "./removeColliderVisual";

const wireframeMaterial = new MeshBasicMaterial({
  color: 0x000000,
  wireframe: true,
});

const tempVector3 = new Vector3();
const tempQuaternion = new Quaternion();

export function createColliderVisual(
  nodeId: string,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  const node = map.nodes.get(nodeId);
  if (!node) throw new Error("Node not found");

  // Remove previous collider
  removeColliderVisual(nodeId, map);

  // Create new collider
  const colliderGroup = new Group();
  let collider: Mesh | null = null;

  switch (node.collider?.type) {
    case "box": {
      collider = new Mesh(
        new BoxGeometry(...node.collider.size),
        wireframeMaterial
      );
      break;
    }

    case "sphere": {
      collider = new Mesh(
        new SphereGeometry(node.collider.radius),
        wireframeMaterial
      );
      break;
    }

    case "cylinder": {
      collider = new Mesh(
        new CylinderGeometry(
          node.collider.radius,
          node.collider.radius,
          node.collider.height,
          32
        ),
        wireframeMaterial
      );
      break;
    }

    case "hull":
    case "mesh": {
      const object = map.objects.get(nodeId);
      if (!object) break;

      if (!node.collider.meshId) break;
      const colliderMesh = map.meshes.get(node.collider.meshId);
      if (!colliderMesh) throw new Error("Collider mesh not found");

      // Get the collider mesh geometry
      const geometries: BufferGeometry[] = [];
      switch (colliderMesh.type) {
        case "Box": {
          geometries.push(
            new BoxGeometry(
              colliderMesh.width,
              colliderMesh.height,
              colliderMesh.depth
            )
          );
          break;
        }

        case "Sphere": {
          geometries.push(
            new SphereGeometry(
              colliderMesh.radius,
              colliderMesh.widthSegments,
              colliderMesh.heightSegments
            )
          );
          break;
        }

        case "Cylinder": {
          geometries.push(
            new CylinderGeometry(
              colliderMesh.radius,
              colliderMesh.radius,
              colliderMesh.height,
              colliderMesh.radialSegments
            )
          );
          break;
        }

        case "Primitives": {
          colliderMesh.primitives.forEach((primitive) => {
            const primitiveObject = map.objects.get(primitive.id);
            if (!primitiveObject) return;

            primitiveObject.traverse((child) => {
              if (child instanceof Mesh) {
                const cloned = child.geometry.clone();
                cloned.applyMatrix4(child.matrixWorld);
                geometries.push(cloned);
              }
            });
          });
          break;
        }
      }

      if (geometries.length === 0) break;

      // Merge geometries
      const geometry = mergeBufferGeometries(geometries);

      // Remove indices if hull
      if (node.collider.type === "hull") geometry.deleteAttribute("index");

      // Create mesh
      collider = new Mesh(geometry, wireframeMaterial);
      collider.position.sub(object.getWorldPosition(tempVector3));
      collider.quaternion.multiply(
        object.getWorldQuaternion(tempQuaternion).invert()
      );

      // Send geometry attributes to physics thread
      const attribute = geometry.getAttribute("position");
      const positions = Float32Array.from(attribute.array);
      const indices = geometry.index
        ? Uint32Array.from(geometry.index.array)
        : undefined;

      postMessage({
        subject: "set_collider_geometry",
        data: {
          nodeId,
          positions,
          indices,
        },
      });
      break;
    }
  }

  if (collider) {
    const object = map.objects.get(nodeId);
    if (!object) throw new Error("Object not found");

    // Add collider to scene
    colliderGroup.add(collider);
    map.colliders.set(nodeId, colliderGroup);
    visuals.add(colliderGroup);

    // Update collider group position
    const globalPosition = object.getWorldPosition(tempVector3);
    const globalQuaternion = object.getWorldQuaternion(tempQuaternion);

    colliderGroup.position.copy(globalPosition);
    colliderGroup.quaternion.copy(globalQuaternion);
  }
}
