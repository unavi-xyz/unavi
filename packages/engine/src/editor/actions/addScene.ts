import { nanoidShort } from "../../client/utils/nanoid";
import { syncedStore } from "../store";

export function addScene(name = "") {
  const id = nanoidShort();

  syncedStore.scenes[id] = {
    id,
    name,
  };

  return id;
}
