import { useClientStore } from "@unavi/react-client";
import { nanoid } from "nanoid";
import { BoxGeometry } from "three";

import { addThreeGeometry } from "./helpers/addThreeGeometry";

export function addBox() {
  const geometry = new BoxGeometry();

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
