import { Bone, Group, Mesh, Quaternion, Vector3 } from "three";

import { Entity } from "../../../scene";
import { RenderScene } from "../../RenderScene";
import { defaultMaterial } from "../constants";
import { removeEntityObject } from "../remove/removeEntityObject";
import { SceneMap } from "../types";
import { updateGlobalTransform } from "../utils/updateGlobalTransform";
import { createColliderVisual } from "./createColliderVisual";
import { createMesh } from "./createMesh";

const tempVector = new Vector3();
const tempQuaternion = new Quaternion();
const tempQuaternion2 = new Quaternion();

export function createEntity(
  entity: Entity,
  map: SceneMap,
  scene: RenderScene,
  visuals: Group
) {
  if (entity.id === "root") return;

  // If no parent, create parent first
  const parentObject = map.objects.get(entity.parentId);
  if (!parentObject) {
    if (!entity.parent) throw new Error("Parent not found");
    createEntity(entity.parent, map, scene, visuals);
  }

  // If skinned mesh, load all joints first
  if (entity.mesh?.type === "Primitive" && entity.mesh.skin !== null) {
    entity.mesh.skin.jointIds.forEach((jointId) => {
      const joint = scene.entities[jointId];
      if (!joint) throw new Error("Joint not found");

      const jointObject = map.objects.get(jointId);
      if (!jointObject) createEntity(joint, map, scene, visuals);

      if (!(jointObject instanceof Bone))
        createMesh(jointId, joint.mesh?.toJSON(), map, scene, visuals);
    });
  }

  createMesh(entity.id, entity.mesh?.toJSON(), map, scene, visuals);

  // Subscribe to entity updates
  entity.mesh$.subscribe({
    next: (mesh) => {
      createMesh(entity.id, mesh?.toJSON(), map, scene, visuals);
    },
    complete: () => {
      // Remove object when entity is removed
      // Nothing special about this callback, could be added to any of these subscriptions
      removeEntityObject(entity.id, map);
    },
  });

  entity.parentId$.subscribe({
    next: (parentId) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error(`Object not found: ${entity.id}`);

      const parentObject = map.objects.get(parentId);
      if (!parentObject) throw new Error(`Parent not found: ${parentId}`);

      // Save object transform
      const position = object.getWorldPosition(tempVector);
      const quaternion = object.getWorldQuaternion(tempQuaternion);

      // Set parent
      parentObject.add(object);

      // Restore object transform
      const inverseParentRotation = parentObject
        .getWorldQuaternion(tempQuaternion2)
        .invert();
      object.position.copy(parentObject.worldToLocal(position));
      object.quaternion.multiplyQuaternions(quaternion, inverseParentRotation);

      // Subscribe to global transform changes
      const parent = scene.entities[parentId];
      if (!parent) throw new Error("Parent not found");

      parent.globalPosition$.subscribe({
        next: () => {
          updateGlobalTransform(entity.id, map, scene);
        },
      });

      parent.globalQuaternion$.subscribe({
        next: () => {
          updateGlobalTransform(entity.id, map, scene);
        },
      });
    },
  });

  entity.materialId$.subscribe({
    next: (materialId) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error("Object not found");

      // Can only set material on meshes
      if (!(object instanceof Mesh)) {
        if (materialId === null) return;
        throw new Error("Object is not a mesh");
      }

      // Get material
      const material = materialId
        ? map.materials.get(materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      // Set material
      object.material = material;

      // Update material on certain mesh changes
      // TODO: Create a new material if the mesh needs to modify it
      entity.mesh$.subscribe({
        next: (mesh) => {
          if (mesh?.type !== "Primitive") return;

          const meshObject = map.objects.get(entity.id);
          if (!meshObject) throw new Error("Mesh not found");
          if (!(meshObject instanceof Mesh))
            throw new Error("Object is not a mesh");

          // Occlusion map needs a second set of UVs
          if (
            meshObject.material.aoMap &&
            meshObject.geometry.attributes.uv &&
            !meshObject.geometry.attributes.uv2
          ) {
            meshObject.geometry.setAttribute(
              "uv2",
              meshObject.geometry.attributes.uv
            );
          }

          // Enable flat shading if no normal attribute
          if (!meshObject.geometry.attributes.normal)
            meshObject.material.flatShading = true;

          // Enable vertex colors if color attribute
          if (meshObject.geometry.attributes.color)
            meshObject.material.vertexColors = true;

          // If three.js needs to generate tangents, flip normal map y
          if (!meshObject.geometry.attributes.tangent)
            meshObject.material.normalScale.y *= -1;
        },
      });
    },
  });

  entity.position$.subscribe({
    next: (position) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error("Object not found");

      // Translate object
      object.position.fromArray(position);

      updateGlobalTransform(entity.id, map, scene);
    },
  });

  entity.rotation$.subscribe({
    next: (rotation) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error("Object not found");

      // Rotate object
      object.quaternion.fromArray(rotation);

      updateGlobalTransform(entity.id, map, scene);
    },
  });

  entity.scale$.subscribe({
    next: (scale) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error("Object not found");

      // Scale object
      object.scale.fromArray(scale);
    },
  });

  entity.collider$.subscribe({
    next: () => {
      createColliderVisual(entity.id, map, scene, visuals);
    },
  });
}
