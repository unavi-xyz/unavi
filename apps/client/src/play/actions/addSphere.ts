import { useClientStore } from "@unavi/engine";
import { SphereGeometry } from "three";

import { addNode } from "./addNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addSphere() {
  const geometry = new SphereGeometry(0.5);

  const mesh = addThreeMesh(geometry);
  const name = addNode("Sphere");

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
