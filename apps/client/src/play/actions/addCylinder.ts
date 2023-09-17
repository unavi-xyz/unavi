import { addNode, editNode, useSceneStore } from "@unavi/engine";
import { CylinderGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";
import { addToRoot } from "./utils/addToRoot";

export function addCylinder() {
  const geometry = new CylinderGeometry(0.5, 0.5);
  const meshId = addThreeMesh(geometry, "Cylinder");

  const { sceneTreeId } = useSceneStore.getState();

  const id = addNode("Cylinder");

  editNode(id, {
    meshId,
    parentId: sceneTreeId,
  });

  if (!sceneTreeId) {
    addToRoot(id);
  }

  return id;
}
