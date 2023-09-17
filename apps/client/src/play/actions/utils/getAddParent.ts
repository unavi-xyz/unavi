import { useSceneStore } from "@unavi/engine";

export function getAddParent(): string {
  const { rootId, sceneTreeId } = useSceneStore.getState();
  return sceneTreeId || rootId || "";
}
