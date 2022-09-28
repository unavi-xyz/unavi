import { Bone, Group } from "three";

import { Entity } from "../../scene";
import { RenderScene } from "../RenderScene";
import { createColliderVisual } from "./createColliderVisual";
import { createMesh } from "./createMesh";
import { moveEntity } from "./moveEntity";
import { SceneMap } from "./SceneMap";
import { setMaterial } from "./setMaterial";
import { updateGlobalTransform } from "./updateGlobalTransform";

export function createEntity(
  entity: Entity,
  map: SceneMap,
  scene: RenderScene,
  visuals: Group
) {
  if (entity.id === "root") return;

  const parentObject = map.objects.get(entity.parentId);

  // If no parent, create parent first
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
      if (!jointObject)
        createMesh(jointId, joint.mesh?.toJSON(), map, scene, visuals);
      if (!(jointObject instanceof Bone))
        createEntity(joint, map, scene, visuals);
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
      // removeEntityObject(entity.id, map);
    },
  });

  entity.parentId$.subscribe({
    next: (parentId) => {
      moveEntity(entity.id, parentId, map, scene);
    },
  });

  entity.materialId$.subscribe({
    next: (materialId) => {
      setMaterial(entity.id, materialId, map);
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
      object.rotation.fromArray(rotation);

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
