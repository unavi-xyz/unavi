import { GlobalTransform, Name, Parent, Transform } from "houseki/scene";
import { Commands, EventReader } from "thyseus";

import { EditorId } from "../../client/components";
import { AddNode } from "../events";

export function addNodes(commands: Commands, events: EventReader<AddNode>) {
  if (events.length === 0) return;

  for (const e of events) {
    commands
      .spawn(true)
      .add(new EditorId(e.id))
      .addType(Name)
      .addType(Parent)
      .addType(Transform)
      .addType(GlobalTransform).id;
  }

  events.clear();
}
