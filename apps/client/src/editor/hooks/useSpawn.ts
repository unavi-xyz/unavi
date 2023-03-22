import { SPAWN_TITLE } from "@wired-labs/gltf-extensions";

import { useEditorStore } from "../store";

export function useSpawn(type: (typeof SPAWN_TITLE)[keyof typeof SPAWN_TITLE] = "Default") {
  const engine = useEditorStore((state) => state.engine);

  const spawn = engine?.scene.getSpawn(type);

  return spawn;
}
