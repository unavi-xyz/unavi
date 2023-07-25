import { useClientStore } from "@unavi/engine";
import { CylinderGeometry } from "three";

import { addNode } from "./addNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addCylinder() {
  const geometry = new CylinderGeometry(0.5, 0.5);

  const mesh = addThreeMesh(geometry);
  const name = addNode("Cylinder");

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
