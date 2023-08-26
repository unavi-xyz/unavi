import { connectionStore } from "@unavi/engine";
import { AddMesh, EditorEvent } from "@unavi/protocol";

import { genName } from "./utils/genName";

export function addMesh(namePrefix = "Mesh") {
  const name = genName(namePrefix);

  const addMesh = AddMesh.create({
    id: name,
  });
  const event = EditorEvent.create({
    event: { addMesh, oneofKind: "addMesh" },
  });

  connectionStore.mirrorEvent(event);

  return name;
}
