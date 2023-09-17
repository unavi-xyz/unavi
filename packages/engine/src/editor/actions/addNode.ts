import { nanoidShort } from "../../client/utils/nanoid";
import {
  SyncedNode_Collider_Type,
  SyncedNode_RigidBody_Type,
  syncedStore,
} from "../store";

export function addNode(name = "") {
  const id = nanoidShort();

  syncedStore.nodes[id] = {
    collider: {
      height: 1,
      meshId: "",
      radius: 1,
      size: [1, 1, 1],
      type: SyncedNode_Collider_Type.NONE,
    },
    id,
    locked: false,
    name: name || "Node",
    rigidBody: {
      type: SyncedNode_RigidBody_Type.STATIC,
    },
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
    translation: [0, 0, 0],
  };

  return id;
}
