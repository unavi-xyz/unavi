import { nanoidShort } from "../../client/utils/nanoid";
import { syncedStore } from "../store";

export function addMesh(name = "") {
  const id = nanoidShort();

  syncedStore.meshes[id] = {
    id,
    indices: [],
    name: name || "Mesh",
    normals: [],
    positions: [],
    uv: [],
  };

  return id;
}
