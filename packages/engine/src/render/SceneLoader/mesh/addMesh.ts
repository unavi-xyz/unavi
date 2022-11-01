import { Group } from "three";

import { MeshJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { SceneMap } from "../types";
import { updateMesh } from "./updateMesh";

export function addMesh(
  mesh: MeshJSON,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  map.meshes.set(mesh.id, mesh);
  updateMesh(mesh.id, mesh, map, visuals, postMessage);
}
