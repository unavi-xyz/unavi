import { Engine, TreeItem } from "@wired-xr/new-engine";

import { useStudioStore } from "../store";

export function updateTree() {
  const treeNonce = useStudioStore.getState().treeNonce;
  useStudioStore.setState({ treeNonce: treeNonce + 1 });
}

export function findItem(
  key: string,
  item: TreeItem,
  property: keyof TreeItem = "id"
): TreeItem | undefined {
  if (item[property] === key) return item;

  // Children
  for (const child of item.children) {
    const found = findItem(key, child, property);
    if (found) return found;
  }
}

export function addItemAsSibling(
  item: TreeItem,
  sibling: TreeItem,
  placement: "above" | "below" = "below"
) {
  const parent = sibling.parent;
  if (!parent) return;

  // Add item to parent
  parent.addChild(item);
  const siblingIndex = parent.children.indexOf(sibling);
  const itemIndex = parent.children.indexOf(item);

  // Remove item from parent children array
  parent.children.splice(itemIndex, 1);

  // Add it back in at the correct index
  if (placement === "above") {
    parent.children.splice(siblingIndex, 0, item);
  } else {
    parent.children.splice(siblingIndex + 1, 0, item);
  }

  updateTree();
}

export async function cloneItem(item: TreeItem, engine: Engine) {
  const clone = new TreeItem();
  clone.name = item.name;

  // Three object
  if (item.threeUUID) {
    const object = await engine.renderThread.getObject(item.threeUUID);
    const clonedObject = object.clone();
    clone.threeUUID = clonedObject.uuid;
    engine.renderThread.addObject(clonedObject);
  }

  // Children
  const childrenPromises = item.children.map((child) => cloneItem(child, engine));
  clone.children = await Promise.all(childrenPromises);

  return clone;
}

export function removeItem(item: TreeItem) {
  item.removeFromParent();

  if (item.threeUUID) {
    const engine = useStudioStore.getState().engine;
    if (engine) {
      engine.renderThread.removeObject(item.threeUUID);
    }
  }

  updateTree();
}

export function moveItem(item: TreeItem, parent: TreeItem) {
  parent.addChild(item);

  if (item.threeUUID && parent.threeUUID) {
    const engine = useStudioStore.getState().engine;
    if (engine) {
      engine.renderThread.moveObject(item.threeUUID, parent.threeUUID);
    }
  }

  updateTree();
}
