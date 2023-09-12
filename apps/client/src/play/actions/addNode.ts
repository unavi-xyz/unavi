import { useClientStore } from "@unavi/engine";
import { AddNode, EditorEvent } from "@unavi/protocol";

import { nanoidShort } from "@/src/server/nanoid";

import { editNode } from "./editNode";
import { genName } from "./utils/genName";

export function addNode(namePrefix = "Node") {
  const name = genName(namePrefix);
  const id = nanoidShort();

  const addNode = AddNode.create({
    id,
  });
  const event = EditorEvent.create({
    event: { addNode, oneofKind: "addNode" },
  });
  useClientStore.getState().sendEditorEvent(event);

  editNode(id, {
    name,
  });

  return id;
}
