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

import { EntityJSON } from "../../../scene";
import { PostMessage, Transferable } from "../../../types";
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
  entityId: string,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  const entity = map.entities.get(entityId);
  if (!entity) throw new Error("Entity not found");

  // Remove previous collider
  removeColliderVisual(entityId, map);

  // Create new collider
  const colliderGroup = new Group();
  let collider: Mesh | null = null;

  switch (entity.collider?.type) {
    case "box": {
      collider = new Mesh(
        new BoxGeometry(...entity.collider.size),
        wireframeMaterial
      );
      break;
    }

    case "sphere": {
      collider = new Mesh(
        new SphereGeometry(entity.collider.radius),
        wireframeMaterial
      );
      break;
    }

    case "cylinder": {
      collider = new Mesh(
        new CylinderGeometry(
          entity.collider.radius,
          entity.collider.radius,
          entity.collider.height,
          32
        ),
        wireframeMaterial
      );
      break;
    }

    case "mesh":
    case "hull": {
      const object = map.objects.get(entityId);
      if (!object) break;

      const geometries: BufferGeometry[] = [];

      if (object instanceof Mesh) {
        const cloned = object.geometry.clone();
        cloned.applyMatrix4(object.matrixWorld);
        geometries.push(cloned);
      }

      const primitives: EntityJSON[] = [];

      map.entities.forEach((entity) => {
        if (
          entity.mesh?.type === "Primitive" &&
          entity.mesh.gltfId === entityId
        ) {
          primitives.push(entity);
        }
      });

      primitives.forEach((primitive) => {
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

      if (geometries.length === 0) break;

      const geometry = mergeBufferGeometries(geometries);

      if (entity.collider?.type === "hull") {
        // Remove indices
        geometry.deleteAttribute("index");
      }

      collider = new Mesh(geometry, wireframeMaterial);
      collider.position.sub(object.getWorldPosition(tempVector3));
      collider.quaternion.multiply(
        object.getWorldQuaternion(tempQuaternion).invert()
      );

      // Send positions to physics thread
      const attribute = geometry.getAttribute("position");
      const positions = Float32Array.from(attribute.array);

      const indices = geometry.index
        ? Uint32Array.from(geometry.index.array)
        : undefined;

      const transfer: Transferable[] = [positions.buffer];
      if (indices) transfer.push(indices.buffer);

      postMessage(
        {
          subject: "set_collider_geometry",
          data: {
            entityId,
            positions,
            indices,
          },
        },
        transfer
      );
      break;
    }
  }

  if (collider) {
    const object = map.objects.get(entityId);
    if (!object) throw new Error("Object not found");

    colliderGroup.add(collider);

    // Add new collider
    map.colliders.set(entityId, colliderGroup);
    visuals.add(colliderGroup);

    // Update collider position
    const globalPosition = object.getWorldPosition(tempVector3);
    const globalQuaternion = object.getWorldQuaternion(tempQuaternion);

    colliderGroup.position.copy(globalPosition);
    colliderGroup.quaternion.copy(globalQuaternion);
  }
}
