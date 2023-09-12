import { useClientStore } from "@unavi/engine";
import { EditNode, EditorEvent } from "@unavi/protocol";

export function editNode(
  id: string,
  data: Parameters<typeof EditNode.create>[0]
) {
  const editNode = EditNode.create({
    target: id,
    ...data,
  });
  const event = EditorEvent.create({
    event: { editNode, oneofKind: "editNode" },
  });
  useClientStore.getState().sendEditorEvent(event);
}
