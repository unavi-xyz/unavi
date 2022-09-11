import { moveEntity } from "../actions/MoveEntityAction";
import { useEditorStore } from "../store";

export function moveToSibling(
  id: string,
  siblingId: string,
  placement: "above" | "below" = "below"
) {
  // Get parent
  const { scene } = useEditorStore.getState();
  const sibling = scene.entities[siblingId];
  const parentId = sibling.parent;

  // Get target index
  let index: number | undefined;

  if (parentId) {
    const parent = scene.entities[parentId];
    index = parent.children.indexOf(siblingId);
    if (placement === "below") index++;
  }

  // Move entity
  moveEntity(id, parentId, index);
}
