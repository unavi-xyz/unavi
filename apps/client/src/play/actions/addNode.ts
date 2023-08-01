import { useClientStore } from "@unavi/engine";
import { AddNode } from "@unavi/protocol";

import { genName } from "./utils/genName";

export function addNode(namePrefix = "Node") {
  const name = genName(namePrefix);

  const event = AddNode.create({
    id: name,
  });
  useClientStore.getState().mirrorEvent(AddNode.toBinary(event));

  return name;
}
