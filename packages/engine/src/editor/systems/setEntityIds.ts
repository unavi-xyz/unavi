import { Entity, Query } from "thyseus";

import { EditorId } from "../../client/components";
import { setEntityId } from "../entities";

export function setEntityIds(ids: Query<[Entity, EditorId]>) {
  for (const [ent, id] of ids) {
    setEntityId(id.value, ent.id);
  }
}
