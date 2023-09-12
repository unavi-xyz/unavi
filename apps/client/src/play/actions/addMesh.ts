import { useClientStore } from "@unavi/engine";
import { AddMesh, EditorEvent } from "@unavi/protocol";

import { nanoidShort } from "@/src/server/nanoid";

import { editMesh } from "./editMesh";
import { genName } from "./utils/genName";

export function addMesh(namePrefix = "Mesh") {
  const id = nanoidShort();
  const name = genName(namePrefix);

  const addMesh = AddMesh.create({
    id,
  });
  const event = EditorEvent.create({
    event: { addMesh, oneofKind: "addMesh" },
  });
  useClientStore.getState().sendEditorEvent(event);

  editMesh(id, { name });

  return id;
}
