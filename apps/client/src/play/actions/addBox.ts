import { useClientStore } from "@unavi/engine";
import { BoxGeometry } from "three";

import { addNode } from "./addNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addBox() {
  const geometry = new BoxGeometry();

  const mesh = addThreeMesh(geometry);
  const name = addNode("Box");

  const parent = getAddParent();

  useClientStore.getState().mirrorEvent({
    data: {
      mesh,
      parent,
      target: name,
    },
    id: "xyz.unavi.editor.edit.node",
    target: "client",
  });

  return name;
}
