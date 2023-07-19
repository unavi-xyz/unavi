import {
  GlobalTransform,
  Mesh,
  Name,
  Parent,
  Transform,
} from "lattice-engine/scene";
import { Commands, dropStruct, Entity, EventReader, Mut, Query } from "thyseus";

import { AddNode } from "../events";

export function addNodes(
  commands: Commands,
  addNode: EventReader<AddNode>,
  named: Query<[Entity, Name]>,
  meshes: Query<[Name, Mut<Mesh>]>
) {
  if (addNode.length === 0) return;

  const nameComp = new Name();
  const meshComp = new Mesh();
  const parentComp = new Parent();

  for (const { name, meshName, parentName } of addNode) {
    nameComp.value = name;

    parentComp.id = 0n;

    for (const [entity, name] of named) {
      if (name.value === parentName) {
        parentComp.id = entity.id;
        break;
      }
    }

    const nodeId = commands
      .spawn(true)
      .add(nameComp)
      .add(parentComp)
      .addType(Transform)
      .addType(GlobalTransform).id;

    for (const [name, mesh] of meshes) {
      if (name.value === meshName) {
        mesh.parentId = nodeId;
      }
    }
  }

  dropStruct(nameComp);
  dropStruct(meshComp);
  dropStruct(parentComp);

  addNode.clear();
}
