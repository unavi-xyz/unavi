import { moveEntity } from "../actions/MoveEntityAction";
import { useStudioStore } from "../store";

export function updateTree() {
  const treeNonce = useStudioStore.getState().treeNonce;
  useStudioStore.setState({ treeNonce: treeNonce + 1 });
}

export function moveToSibling(
  id: string,
  siblingId: string,
  placement: "above" | "below" = "below"
) {
  // Get parent
  const tree = useStudioStore.getState().tree;
  const sibling = tree[siblingId];
  const parentId = sibling.parent;

  // Move entity
  moveEntity(id, parentId);
}

export function findChildren(id: string) {
  const tree = useStudioStore.getState().tree;
  if (id === "root")
    return Object.keys(tree).filter((key) => tree[key].parent === null);
  return Object.keys(tree).filter((key) => tree[key].parent === id);
}
