import { useClientStore } from "@unavi/engine";
import { AddMesh } from "@unavi/protocol";

import { genName } from "./utils/genName";

export function addMesh(namePrefix = "Mesh") {
  const name = genName(namePrefix);

  const event = AddMesh.create({
    id: name,
  });
  useClientStore.getState().mirrorEvent(AddMesh.toBinary(event));

  return name;
}
