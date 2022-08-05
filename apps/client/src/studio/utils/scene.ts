import { Object3D } from "three";

import { useStudioStore } from "../store";
import { UserData } from "../types";

export function updateTree() {
  const treeNonce = useStudioStore.getState().treeNonce;
  useStudioStore.setState({ treeNonce: treeNonce + 1 });
}

export function addObjectToScene(object: Object3D) {
  const root = useStudioStore.getState().root;

  const userData: UserData = object.userData;
  userData.treeNode = true;

  root.add(object);

  updateTree();
}

export function removeObjectFromScene(object: Object3D) {
  const root = useStudioStore.getState().root;
  root.remove(object);
  updateTree();
}

export function findObject(uuid: string) {
  const root = useStudioStore.getState().root;

  let found: Object3D | undefined;

  root.traverse((object) => {
    if (object.uuid === uuid) {
      found = object;
    }
  });

  return found;
}

export function addObjectToParent(object: Object3D, parent: Object3D) {
  parent.add(object);
  updateTree();
}

export function addObjectAsSibling(
  object: Object3D,
  sibling: Object3D,
  placement: "above" | "below" = "below"
) {
  const parent = sibling.parent;
  if (!parent) return;

  // Add object to parent
  parent.add(object);

  const siblingIndex = parent.children.indexOf(sibling);
  const objectIndex = parent.children.indexOf(object);

  // Remove object from parent children array
  parent.children.splice(objectIndex, 1);

  // Add it back in at the correct index
  if (placement === "above") {
    parent.children.splice(siblingIndex, 0, object);
  } else {
    parent.children.splice(siblingIndex + 1, 0, object);
  }

  updateTree();
}
