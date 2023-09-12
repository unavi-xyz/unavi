import { CylinderGeometry } from "three";

import { addNode } from "./addNode";
import { editNode } from "./editNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addCylinder() {
  const geometry = new CylinderGeometry(0.5, 0.5);
  const mesh = addThreeMesh(geometry);
  const parent = getAddParent();

  const id = addNode("Cylinder");

  editNode(id, {
    mesh,
    parent,
  });

  return id;
}
