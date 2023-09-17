import { addNode, editNode } from "@unavi/engine";
import { SphereGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addSphere() {
  const geometry = new SphereGeometry(0.5);
  const meshId = addThreeMesh(geometry);
  const parentId = getAddParent();

  const id = addNode("Sphere");

  editNode(id, {
    meshId,
    parentId,
  });

  return id;
}
