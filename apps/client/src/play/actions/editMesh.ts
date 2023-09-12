import { useClientStore } from "@unavi/engine";
import { EditMesh, EditorEvent } from "@unavi/protocol";

export function editMesh(
  id: string,
  data: Parameters<typeof EditMesh.create>[0]
) {
  const editMesh = EditMesh.create({
    target: id,
    ...data,
  });
  const event = EditorEvent.create({
    event: { editMesh, oneofKind: "editMesh" },
  });
  useClientStore.getState().sendEditorEvent(event);
}
