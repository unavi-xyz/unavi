import { addNode, editNode } from "@unavi/engine";
import { BoxGeometry } from "three";

import { addThreeMesh } from "./utils/addThreeMesh";
import { getAddParent } from "./utils/getAddParent";

export function addBox() {
  const geometry = new BoxGeometry();
  const meshId = addThreeMesh(geometry);
  const parentId = getAddParent();

  const id = addNode("Box");

  editNode(id, {
    meshId,
    parentId,
  });

  return id;
}
