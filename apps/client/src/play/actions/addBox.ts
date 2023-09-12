import { BoxGeometry } from "three";

import { addNode } from "./addNode";
import { editNode } from "./editNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addBox() {
  const geometry = new BoxGeometry();
  const mesh = addThreeMesh(geometry);
  const parent = getAddParent();

  const id = addNode("Box");

  editNode(id, {
    mesh,
    parent,
  });

  return id;
}
