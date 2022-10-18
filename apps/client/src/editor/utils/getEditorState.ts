import { useEditorStore } from "../store";

export function getEditorState() {
  const { visuals, tool } = useEditorStore.getState();
  return { visuals, tool };
}
