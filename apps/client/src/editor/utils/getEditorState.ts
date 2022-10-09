import { useEditorStore } from "../store";

export function getEditorState() {
  const { colliders } = useEditorStore.getState();
  return { colliders };
}
