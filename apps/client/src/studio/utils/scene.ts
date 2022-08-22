import { useStudioStore } from "../store";

export function updateTree() {
  const treeNonce = useStudioStore.getState().treeNonce;
  useStudioStore.setState({ treeNonce: treeNonce + 1 });
}

export function getObject(uuid: string) {
  // const object = root.getObjectByProperty("uuid", uuid);
  // return object;
}

export function addItemAsSibling(
  object: any,
  sibling: any,
  placement: "above" | "below" = "below"
) {
  const parent = sibling.parent;
  if (!parent) return;

  // Add object to parent
  moveObject(object, parent);

  const siblingIndex = parent.children.indexOf(sibling);
  const objectIndex = parent.children.indexOf(object);
  const offset = objectIndex < siblingIndex ? -1 : 0;

  // Remove object from parent children array
  parent.children.splice(objectIndex, 1);

  // Add it back in at the correct index
  if (placement === "above") {
    parent.children.splice(siblingIndex + offset, 0, object);
  } else {
    parent.children.splice(siblingIndex + offset + 1, 0, object);
  }

  updateTree();
}

export function moveObject(object: any, newParent: any) {
  // // Save object transform
  // const position = object.getWorldPosition(new Vector3());
  // const rotation = object.getWorldQuaternion(new Quaternion());
  // // Add object to new parent
  // newParent.add(object);
  // // Restore object transform
  // const inverseParentRotation = newParent.getWorldQuaternion(new Quaternion()).invert();
  // object.position.copy(newParent.worldToLocal(position));
  // object.quaternion.multiplyQuaternions(rotation, inverseParentRotation);
}
