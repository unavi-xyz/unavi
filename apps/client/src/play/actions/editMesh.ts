import { useClientStore } from "@unavi/engine";
import { EditMeshPrimitive, EditorEvent } from "@unavi/protocol";

export function editMesh(
  id: string,
  data: Parameters<typeof EditMeshPrimitive.create>[0]
) {
  const editMeshPrimitive = EditMeshPrimitive.create({
    target: id,
    ...data,
  });
  const event = EditorEvent.create({
    event: { editMeshPrimitive, oneofKind: "editMeshPrimitive" },
  });
  useClientStore.getState().sendEditorEvent(event);
}
