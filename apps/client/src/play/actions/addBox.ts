import { addNode, editNode, useSceneStore } from "@unavi/engine";
import { BoxGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";
import { addToRoot } from "./utils/addToRoot";

export function addBox() {
  const geometry = new BoxGeometry();
  const meshId = addThreeMesh(geometry, "Box");

  const { sceneTreeId } = useSceneStore.getState();

  const id = addNode("Box");

  editNode(id, {
    meshId,
    parentId: sceneTreeId,
  });

  if (!sceneTreeId) {
    addToRoot(id);
  }

  return id;
}
