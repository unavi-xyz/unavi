import { connectionStore } from "@unavi/engine";
import { EditNode, EditorEvent } from "@unavi/protocol";

export function editNode(data: Parameters<typeof EditNode.create>[0]) {
  const editNode = EditNode.create(data);
  const event = EditorEvent.create({
    event: { editNode, oneofKind: "editNode" },
  });

  connectionStore.mirrorEvent(event);
}
