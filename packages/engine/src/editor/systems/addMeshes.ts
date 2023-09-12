import { Geometry, Mesh, Name } from "houseki/scene";
import { Commands, EventReader } from "thyseus";

import { EditorId } from "../../client/components";
import { AddMesh } from "../events";

export function addMeshes(commands: Commands, events: EventReader<AddMesh>) {
  if (events.length === 0) return;

  for (const e of events) {
    commands
      .spawn(true)
      .add(new EditorId(e.id))
      .addType(Name)
      .addType(Mesh)
      .addType(Geometry).id;
  }

  events.clear();
}
