import { useStudioStore } from "../store";

export function getStudioState() {
  const { debug, grid, tool, names } = useStudioStore.getState();
  return { debug, grid, tool, names };
}
