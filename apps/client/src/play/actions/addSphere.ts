import { useClientStore } from "@unavi/react-client";
import { SphereGeometry } from "three";

import { addNode } from "./addNode";
import { addThreeMesh } from "./utils/addThreeMesh";

export function addSphere() {
  const geometry = new SphereGeometry(0.5);

  const mesh = addThreeMesh(geometry);
  const name = addNode("Sphere");

  useClientStore.getState().mirrorEvent({
    data: {
      mesh,
      parent: useClientStore.getState().rootName,
      target: name,
    },
    id: "xyz.unavi.editor.edit.node",
    target: "client",
  });
}
