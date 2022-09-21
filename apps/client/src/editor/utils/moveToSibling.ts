import { moveEntity } from "../actions/MoveEntityAction";
import { useEditorStore } from "../store";

export function moveToSibling(
  id: string,
  siblingId: string,
  placement: "above" | "below" = "below"
) {
  const entities = useEditorStore.getState().engine?.scene.entities;
  if (!entities) throw new Error("Entities not found");

  // Get parent
  const parentId = entities[siblingId].parentId;

  // Get target index
  let index: number | undefined;

  if (parentId) {
    const parent = entities[parentId];
    index = parent.childrenIds.indexOf(siblingId);
    if (placement === "below") index++;
  }

  // Move entity
  moveEntity(id, parentId, index);
}
