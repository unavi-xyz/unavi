import { useClientStore } from "@unavi/react-client";
import { nanoid } from "nanoid";
import { CylinderGeometry } from "three";

import { addThreeGeometry } from "./helpers/addThreeGeometry";

export function addCylinder() {
  const geometry = new CylinderGeometry(0.5, 0.5);

  const mesh = addThreeGeometry(geometry);

  useClientStore.getState().mirrorEvent({
    data: {
      mesh,
      name: nanoid(),
      parent: useClientStore.getState().rootName,
    },
    id: "xyz.unavi.editor.add.node",
    target: "client",
  });
}
