import { useEditorStore } from "../store";

export function getEditorState() {
  const { debug, grid, tool } = useEditorStore.getState();
  return { debug, grid, tool };
}
