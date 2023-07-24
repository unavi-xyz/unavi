import { useClientStore } from "@unavi/engine";

import { genName } from "./utils/genName";

export function addMesh(namePrefix = "Mesh") {
  const name = genName(namePrefix);

  useClientStore.getState().mirrorEvent({
    data: { name },
    id: "xyz.unavi.editor.add.mesh",
    target: "client",
  });

  return name;
}
