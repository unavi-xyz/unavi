import { atom } from "jotai";
import { Loading, Warehouse } from "lattice-engine/core";
import { Entity, Query, Res, SystemRes } from "thyseus";

import { AtomStore } from "../../AtomStore";

export class LoadingStore extends AtomStore {
  message = atom("");
  numLoading = atom(0);
  numLoaded = atom(0);
  loadingEntities = atom(new Set<bigint>());
}

export const loadingStore = new LoadingStore();

class LocalRes {
  messageId = 0n;
  message = "";
}

/**
 * Counts the number of entities with a Loading component and updates the loading store.
 */
export function exportLoadingInfo(
  warehouse: Res<Warehouse>,
  localRes: SystemRes<LocalRes>,
  loading: Query<[Entity, Loading]>
) {
  const entityIds = loadingStore.get(loadingStore.loadingEntities);

  const ids: bigint[] = [];

  let displayedId = 0n;
  let displayedMessage = "";

  for (const [entity, load] of loading) {
    ids.push(entity.id);
    entityIds.add(entity.id);

    const message = load.message.read(warehouse);

    if (!displayedMessage && message) {
      displayedId = entity.id;
      displayedMessage = message;
    }
  }

  for (const id of entityIds) {
    if (!ids.includes(id)) {
      entityIds.delete(id);
      loadingStore.set(
        loadingStore.numLoaded,
        loadingStore.get(loadingStore.numLoaded) + 1
      );
    }
  }

  // Only update the message once the current message is done loading
  if (!ids.includes(localRes.messageId)) {
    localRes.messageId = displayedId;
    localRes.message = displayedMessage;
  }

  loadingStore.set(loadingStore.numLoading, ids.length);
  loadingStore.set(loadingStore.message, localRes.message);
}
