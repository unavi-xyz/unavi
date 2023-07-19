import { useClientStore } from "@unavi/react-client";
import { BoxGeometry } from "three";

import { addNode } from "./addNode";
import { addThreeMesh } from "./utils/addThreeMesh";

export function addBox() {
  const geometry = new BoxGeometry();

  const mesh = addThreeMesh(geometry);
  const name = addNode("Box");

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
