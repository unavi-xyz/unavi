import { Group, Mesh, Quaternion, Vector3 } from "three";

import { EntityJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { defaultMaterial } from "../constants";
import { SceneMap } from "../types";
import { updateGlobalTransform } from "../utils/updateGlobalTransform";
import { createColliderVisual } from "./createColliderVisual";
import { createObject } from "./createObject";
import { updateMeshMaterial } from "./updateMeshMaterial";

const tempVector = new Vector3();
const tempQuaternion = new Quaternion();
const tempQuaternion2 = new Quaternion();

export function updateEntity(
  entityId: string,
  data: Partial<EntityJSON>,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  const entity = map.entities.get(entityId);
  if (!entity) throw new Error("Entity not found");

  const newEntity = { ...entity, ...data };
  map.entities.set(entityId, newEntity);

  // Update object
  if (data.mesh !== undefined) createObject(entity, map, visuals, postMessage);

  const object = map.objects.get(entityId);
  if (!object) throw new Error("Object not found");

  // Update parent
  if (data.parentId !== undefined) {
    const parentObject = map.objects.get(data.parentId);
    if (!parentObject) throw new Error("Parent not found");

    // Save object transform
    const position = object.getWorldPosition(tempVector);
    const quaternion = object.getWorldQuaternion(tempQuaternion);

    // Add to parent
    parentObject.add(object);

    // Restore object transform
    const inverseParentRotation = parentObject
      .getWorldQuaternion(tempQuaternion2)
      .invert();
    object.position.copy(parentObject.worldToLocal(position));
    object.quaternion.multiplyQuaternions(quaternion, inverseParentRotation);

    // Update global transform
    updateGlobalTransform(entityId, map, postMessage);
  }

  // Update name
  if (data.name !== undefined) object.name = data.name || entityId;

  // Update transform
  if (data.position !== undefined) object.position.fromArray(data.position);
  if (data.rotation !== undefined) object.quaternion.fromArray(data.rotation);
  if (data.scale !== undefined) object.scale.fromArray(data.scale);

  if (data.position !== undefined || data.rotation !== undefined) {
    updateGlobalTransform(entityId, map, postMessage);
  }

  // Update material
  if (data.materialId !== undefined) {
    if (!(object instanceof Mesh)) {
      if (data.materialId !== null) throw new Error("Object is not a mesh");
    } else {
      const material = data.materialId
        ? map.materials.get(data.materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      object.material = material;

      updateMeshMaterial(entityId, entity.mesh, map);
    }
  }

  // Update collider visual
  if (data.collider) createColliderVisual(entityId, map, visuals);
}
