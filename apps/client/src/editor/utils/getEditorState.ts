import { useEditorStore } from "../store";

export function getEditorState() {
  const { colliders, grid, tool } = useEditorStore.getState();
  return { colliders, grid, tool };
}
