import { Node } from "@gltf-transform/core";
import { SPAWN_TITLE } from "@unavi/gltf-extensions";

import { useStudio } from "../components/Studio";

export function useSpawn(
  type: (typeof SPAWN_TITLE)[keyof typeof SPAWN_TITLE] = "Default"
): Node | undefined {
  const { engine } = useStudio();

  const spawn = engine?.scene.getSpawn(type);

  return spawn;
}
