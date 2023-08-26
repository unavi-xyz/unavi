import { CylinderGeometry } from "three";

import { addNode } from "./addNode";
import { editNode } from "./editNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addCylinder() {
  const geometry = new CylinderGeometry(0.5, 0.5);

  const mesh = addThreeMesh(geometry);
  const name = addNode("Cylinder");

  const parent = getAddParent() ?? undefined;

  editNode({
    mesh,
    parent,
    target: name,
  });

  return name;
}
