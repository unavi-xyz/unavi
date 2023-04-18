import { SPAWN_TITLE } from "@unavi/gltf-extensions";

import { useEditor } from "../components/Editor";

export function useSpawn(type: (typeof SPAWN_TITLE)[keyof typeof SPAWN_TITLE] = "Default") {
  const { engine } = useEditor();

  const spawn = engine?.scene.getSpawn(type);

  return spawn;
}
