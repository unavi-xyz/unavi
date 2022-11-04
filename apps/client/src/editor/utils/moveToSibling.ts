import { moveNode } from "../actions/MoveNodeAction";
import { useEditorStore } from "../store";

export function moveToSibling(
  id: string,
  siblingId: string,
  placement: "above" | "below" = "below"
) {
  const nodes = useEditorStore.getState().engine?.scene.nodes;
  if (!nodes) throw new Error("Nodes not found");

  // Get parent
  const sibling = nodes[siblingId];
  if (!sibling) throw new Error("Sibling not found");
  const parentId = sibling.parentId;

  // Get target index
  let index: number | undefined;

  if (parentId) {
    const parent = nodes[parentId];
    if (!parent) throw new Error("Parent not found");
    index = parent.childrenIds.indexOf(siblingId);
    if (placement === "below") index++;
  }

  // Move node
  moveNode(id, parentId, index);
}
