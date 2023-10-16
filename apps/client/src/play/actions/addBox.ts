import { addNode, editNode, useSceneStore } from "@unavi/engine";
import { BoxGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";

export function addBox() {
  const geometry = new BoxGeometry();
  const meshId = addThreeMesh(geometry, "Box");

  const { sceneTreeId, rootId } = useSceneStore.getState();
  const parentId = sceneTreeId ?? rootId;

  const id = addNode("Box");

  editNode(id, {
    meshId,
    parentId,
  });

  return id;
}
