import { GlobalTransform, Name, Parent, Transform } from "lattice-engine/scene";
import { Commands, EventReader } from "thyseus";

import { AddNode } from "../events";

export function addNodes(commands: Commands, events: EventReader<AddNode>) {
  if (events.length === 0) return;

  const nameComp = new Name();

  for (const e of events) {
    nameComp.value = e.name;

    commands
      .spawn(true)
      .add(nameComp)
      .addType(Parent)
      .addType(Transform)
      .addType(GlobalTransform).id;
  }

  events.clear();
}
