import { addNode, editNode, useSceneStore } from "@unavi/engine";
import { CylinderGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";

export function addCylinder() {
  const geometry = new CylinderGeometry(0.5, 0.5);
  const meshId = addThreeMesh(geometry, "Cylinder");

  const { sceneTreeId, rootId } = useSceneStore.getState();
  const parentId = sceneTreeId ?? rootId;

  const id = addNode("Cylinder");

  editNode(id, {
    meshId,
    parentId,
  });

  return id;
}
