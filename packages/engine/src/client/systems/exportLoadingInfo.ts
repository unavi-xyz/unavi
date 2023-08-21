import { Loading, Warehouse } from "lattice-engine/core";
import { Entity, Query, Res, SystemRes } from "thyseus";
import { create } from "zustand";

type LoadingStore = {
  loading: number;
  loaded: number;
  total: number;
  message: string;
  entityIds: Set<bigint>;
  reset: () => void;
};

export const useLoadingStore = create<LoadingStore>((set) => ({
  entityIds: new Set(),
  loaded: 0,
  loading: 0,
  message: "",
  reset: () => {
    set({
      entityIds: new Set(),
      loaded: 0,
      loading: 0,
      message: "",
      total: 0,
    });
  },
  total: 0,
}));

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
  const entityIds = useLoadingStore.getState().entityIds;

  const ids: bigint[] = [];

  let count = 0;
  let displayedId = 0n;
  let displayedMessage = "";

  for (const [entity, load] of loading) {
    ids.push(entity.id);

    entityIds.add(entity.id);

    count++;

    const message = load.message.read(warehouse);

    if (!displayedMessage && message) {
      displayedId = entity.id;
      displayedMessage = message;
    }
  }

  for (const id of entityIds) {
    if (!ids.includes(id)) {
      entityIds.delete(id);
      useLoadingStore.setState((prev) => ({
        loaded: prev.loaded + 1,
      }));
    }
  }

  // Only update the message once the current message is done loading
  if (!ids.includes(localRes.messageId)) {
    localRes.messageId = displayedId;
    localRes.message = displayedMessage;
  }

  useLoadingStore.setState((prev) => ({
    loading: count,
    message: localRes.message,
    total: prev.loaded + count,
  }));
}
