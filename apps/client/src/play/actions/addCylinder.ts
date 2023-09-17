import { addNode, editNode } from "@unavi/engine";
import { CylinderGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addCylinder() {
  const geometry = new CylinderGeometry(0.5, 0.5);
  const meshId = addThreeMesh(geometry);
  const parentId = getAddParent();

  const id = addNode("Cylinder");

  editNode(id, {
    meshId,
    parentId,
  });

  return id;
}
