import { useClientStore } from "@unavi/engine";
import { AddMeshPrimitive, EditorEvent } from "@unavi/protocol";

import { nanoidShort } from "@/src/server/nanoid";

import { editMesh } from "./editMesh";
import { genName } from "./utils/genName";

export function addMesh(namePrefix = "Mesh") {
  const id = nanoidShort();
  const name = genName(namePrefix);

  const addMeshPrimitive = AddMeshPrimitive.create({
    id,
  });
  const event = EditorEvent.create({
    event: { addMeshPrimitive, oneofKind: "addMeshPrimitive" },
  });
  useClientStore.getState().sendEditorEvent(event);

  editMesh(id, { name });

  return id;
}
