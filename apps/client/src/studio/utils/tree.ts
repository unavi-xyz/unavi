import { addComponent, removeComponent } from "bitecs";

import { Child, rootObjectsQuery } from "@wired-xr/engine";

import { useStudioStore } from "../store";

export function updateTree() {
  const treeNonce = useStudioStore.getState().treeNonce;
  useStudioStore.setState({ treeNonce: treeNonce + 1 });
}

export function addItemAsSibling(
  eid: number,
  sibling: number,
  placement: "above" | "below" = "below"
) {
  const engine = useStudioStore.getState().engine;
  if (!engine) throw new Error("Engine not found");

  const withoutParent = rootObjectsQuery(engine.world);
  const siblingIsRoot = withoutParent.includes(sibling);

  if (siblingIsRoot) {
    removeComponent(engine.world, Child, eid);
    return;
  }

  const parentEid = Child.parent[sibling];
  moveObject(eid, parentEid);
}

export function moveObject(eid: number, parentEid: number) {
  // // Save object transform
  // const position = object.getWorldPosition(new Vector3());
  // const rotation = object.getWorldQuaternion(new Quaternion());

  const engine = useStudioStore.getState().engine;
  if (!engine) throw new Error("Engine not found");

  // If parent is root, remove child component
  if (parentEid === -1) {
    removeComponent(engine.world, Child, eid);
    return;
  }

  // Add child component
  addComponent(engine.world, Child, eid);
  Child.parent[eid] = parentEid;

  // // Restore object transform
  // const inverseParentRotation = newParent.getWorldQuaternion(new Quaternion()).invert();
  // object.position.copy(newParent.worldToLocal(position));
  // object.quaternion.multiplyQuaternions(rotation, inverseParentRotation);
}
