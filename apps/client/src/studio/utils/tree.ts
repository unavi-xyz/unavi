import { moveEntity } from "../actions/MoveEntityAction";
import { useStudioStore } from "../store";

export function moveToSibling(
  id: string,
  siblingId: string,
  placement: "above" | "below" = "below"
) {
  // Get parent
  const { tree } = useStudioStore.getState();
  const sibling = tree[siblingId];
  const parentId = sibling.parent;

  // Get target index
  let index: number | undefined;

  if (parentId) {
    const parent = tree[parentId];
    index = parent.children.indexOf(siblingId);
    if (placement === "below") index++;
  }

  // Move entity
  moveEntity(id, parentId, index);
}
