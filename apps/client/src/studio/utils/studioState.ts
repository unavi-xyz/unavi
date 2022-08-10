import { useStudioStore } from "../store";
import { Tool } from "../types";

interface StudioState {
  debug: boolean;
  grid: boolean;
  tool: Tool;
  selectedId: string | null;
}

export function getStudioState(): StudioState {
  const { debug, grid, tool, selectedId } = useStudioStore.getState();
  return { debug, grid, tool, selectedId };
}
