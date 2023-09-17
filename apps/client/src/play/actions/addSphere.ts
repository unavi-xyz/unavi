import { addNode, editNode, useSceneStore } from "@unavi/engine";
import { SphereGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";
import { addToRoot } from "./utils/addToRoot";

export function addSphere() {
  const geometry = new SphereGeometry(0.5);
  const meshId = addThreeMesh(geometry, "Sphere");

  const { sceneTreeId } = useSceneStore.getState();

  const id = addNode("Sphere");

  editNode(id, {
    meshId,
    parentId: sceneTreeId,
  });

  if (!sceneTreeId) {
    addToRoot(id);
  }

  return id;
}
