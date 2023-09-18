import { SubScene } from "houseki/gltf";
import { DynamicBody, StaticBody } from "houseki/physics";
import { Parent, Transform } from "houseki/scene";
import { Commands, Entity, Query, With, Without } from "thyseus";

import { EditorId } from "../../client/components";
import { SyncedNode_RigidBody_Type, syncedStore } from "../store";

export function createRigidBodies(
  commands: Commands,
  nodes: Query<
    [Entity, EditorId],
    [With<Transform>, With<Parent>, Without<SubScene>]
  >,
  staticBodies: Query<Entity, With<StaticBody>>,
  dynamicBodies: Query<Entity, With<DynamicBody>>
) {
  for (const [entity, id] of nodes) {
    const node = syncedStore.nodes[id.value];
    if (!node) continue;

    let foundStatic = false;
    let foundDynamic = false;

    for (const ent of staticBodies) {
      if (ent.id !== entity.id) continue;

      foundStatic = true;

      if (node.rigidBody.type === SyncedNode_RigidBody_Type.STATIC) continue;

      commands.get(entity).remove(StaticBody);
    }

    for (const ent of dynamicBodies) {
      if (ent.id !== entity.id) continue;

      foundDynamic = true;

      if (node.rigidBody.type === SyncedNode_RigidBody_Type.DYNAMIC) continue;

      commands.get(entity).remove(DynamicBody);
    }

    if (
      !foundStatic &&
      node.rigidBody.type === SyncedNode_RigidBody_Type.STATIC
    ) {
      commands.get(entity).addType(StaticBody);
    }

    if (
      !foundDynamic &&
      node.rigidBody.type === SyncedNode_RigidBody_Type.DYNAMIC
    ) {
      commands.get(entity).addType(DynamicBody);
    }
  }
}
