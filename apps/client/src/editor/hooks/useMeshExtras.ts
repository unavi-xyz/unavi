import { Mesh } from "@gltf-transform/core";
import { MeshExtras } from "engine";

import { useSubscribe } from "./useSubscribe";

export function useMeshExtras(mesh: Mesh | null) {
  return useSubscribe(mesh, "Extras") as MeshExtras | undefined;
}
