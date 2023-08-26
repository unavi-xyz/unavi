import { BoxGeometry } from "three";

import { addNode } from "./addNode";
import { editNode } from "./editNode";
import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addBox() {
  const geometry = new BoxGeometry();

  const mesh = addThreeMesh(geometry);
  const name = addNode("Box");

  const parent = getAddParent() ?? undefined;

  editNode({
    mesh,
    parent,
    target: name,
  });

  return name;
}
