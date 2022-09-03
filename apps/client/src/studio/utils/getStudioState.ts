import { useStudioStore } from "../store";

export function getStudioState() {
  const { debug, grid, tool } = useStudioStore.getState();
  return { debug, grid, tool };
}
