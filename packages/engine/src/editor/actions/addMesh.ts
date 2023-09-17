import { nanoidShort } from "../../client/utils/nanoid";
import { syncedStore } from "../store";

export function addMesh(name = "") {
  const id = nanoidShort();

  syncedStore.meshes[id] = {
    colors: [],
    id,
    indices: [],
    joints: [],
    name: name || "Mesh",
    normals: [],
    positions: [],
    uv: [],
    uv1: [],
    uv2: [],
    uv3: [],
    weights: [],
  };

  return id;
}
