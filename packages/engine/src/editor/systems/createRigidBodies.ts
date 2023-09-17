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

    switch (node.rigidBody.type) {
      case SyncedNode_RigidBody_Type.STATIC: {
        let found = false;

        for (const ent of staticBodies) {
          if (ent.id === entity.id) {
            found = true;
          }
        }

        if (!found) {
          commands.get(entity).remove(DynamicBody).addType(StaticBody);
        }

        break;
      }

      case SyncedNode_RigidBody_Type.DYNAMIC: {
        let found = false;

        for (const ent of dynamicBodies) {
          if (ent.id === entity.id) {
            found = true;
          }
        }

        if (!found) {
          commands.get(entity).remove(StaticBody).addType(DynamicBody);
        }

        break;
      }
    }
  }
}
