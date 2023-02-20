import { SPAWN_TITLE } from "engine";

import { useEditorStore } from "../store";

export function useSpawn(type: (typeof SPAWN_TITLE)[keyof typeof SPAWN_TITLE] = "Default") {
  const engine = useEditorStore((state) => state.engine);

  const spawn = engine?.scene.getSpawn(type);

  return spawn;
}
