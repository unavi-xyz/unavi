import { addNode, editNode, useSceneStore } from "@unavi/engine";
import { SphereGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";

export function addSphere() {
  const geometry = new SphereGeometry(0.5);
  const meshId = addThreeMesh(geometry, "Sphere");

  const { sceneTreeId, rootId } = useSceneStore.getState();
  const parentId = sceneTreeId ?? rootId;

  const id = addNode("Sphere");

  editNode(id, {
    meshId,
    parentId,
  });

  return id;
}
