import { Geometry, Mesh, Name } from "houseki/scene";
import { Commands, EventReader } from "thyseus";

import { AddMesh } from "../events";

export function addMeshes(commands: Commands, events: EventReader<AddMesh>) {
  if (events.length === 0) return;

  const nameComp = new Name();

  for (const e of events) {
    nameComp.value = e.name;

    commands.spawn(true).add(nameComp).addType(Mesh).addType(Geometry).id;
  }

  events.clear();
}
