import { useStudioStore } from "../store";
import { Tool } from "../types";

interface StudioState {
  preview: boolean;
  debug: boolean;
  grid: boolean;
  tool: Tool;
  selectedId: string | null;
}

export function getStudioState(): StudioState {
  const { preview, debug, grid, tool, selectedId } = useStudioStore.getState();
  return { preview, debug, grid, tool, selectedId };
}
