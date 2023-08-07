import { useClientStore } from "@unavi/engine";
import { AddNode, EditorEvent } from "@unavi/protocol";

import { genName } from "./utils/genName";

export function addNode(namePrefix = "Node") {
  const name = genName(namePrefix);

  const addNode = AddNode.create({
    id: name,
  });
  const event = EditorEvent.create({ event: { addNode, oneofKind: "addNode" } });
  useClientStore.getState().mirrorEvent(event);

  return name;
}
