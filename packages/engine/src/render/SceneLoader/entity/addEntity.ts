import { Group } from "three";

import { EntityJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { SceneMap } from "../types";
import { updateEntity } from "./updateEntity";

export function addEntity(
  entity: EntityJSON,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  map.entities.set(entity.id, entity);
  updateEntity(entity.id, entity, map, visuals, postMessage);
}
