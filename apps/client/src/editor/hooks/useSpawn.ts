import { SPAWN_TITLES } from "engine";

import { useEditorStore } from "../store";

export function useSpawn(type: (typeof SPAWN_TITLES)[keyof typeof SPAWN_TITLES] = "Default") {
  const engine = useEditorStore((state) => state.engine);

  const spawn = engine?.scene.getSpawn(type);

  return spawn;
}
