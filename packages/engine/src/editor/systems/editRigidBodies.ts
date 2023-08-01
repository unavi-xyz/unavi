import { EditNode_RigidBody_Type } from "@unavi/protocol";
import { DynamicBody, StaticBody } from "lattice-engine/physics";
import { Name } from "lattice-engine/scene";
import { Commands, Entity, EventReader, Query, With } from "thyseus";

import { EditRigidBody } from "../events";

export function editRigidBodies(
  commands: Commands,
  events: EventReader<EditRigidBody>,
  names: Query<[Entity, Name]>,
  dynamicBodies: Query<Entity, With<DynamicBody>>,
  staticBodies: Query<Entity, With<StaticBody>>
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [entity, name] of names) {
      if (name.value !== e.target) continue;

      // Remove the old rigid body
      for (const rigidBodyEntity of dynamicBodies) {
        if (rigidBodyEntity.id !== entity.id) continue;
        commands.get(entity).remove(DynamicBody);
      }

      for (const rigidBodyEntity of staticBodies) {
        if (rigidBodyEntity.id !== entity.id) continue;
        commands.get(entity).remove(StaticBody);
      }

      // Add the new rigid body
      if (e.type === EditNode_RigidBody_Type.DYNAMIC) {
        commands.get(entity).addType(DynamicBody);
      }

      if (e.type === EditNode_RigidBody_Type.STATIC) {
        commands.get(entity).addType(StaticBody);
      }
    }
  }

  events.clear();
}
