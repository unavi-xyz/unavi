import { SphereGeometry } from "three";

import { addNode } from "./addNode";
import { editNode } from "./editNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addSphere() {
  const geometry = new SphereGeometry(0.5);
  const mesh = addThreeMesh(geometry);
  const parent = getAddParent();

  const id = addNode("Sphere");

  editNode(id, {
    mesh,
    parent,
  });

  return id;
}
