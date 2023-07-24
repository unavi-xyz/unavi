import { useClientStore } from "@unavi/engine";

import { genName } from "./utils/genName";

export function addNode(namePrefix = "Node") {
  const name = genName(namePrefix);

  useClientStore.getState().mirrorEvent({
    data: { name },
    id: "xyz.unavi.editor.add.node",
    target: "client",
  });

  return name;
}
