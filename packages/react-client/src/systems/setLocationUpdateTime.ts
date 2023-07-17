import { Mut, Query } from "thyseus";

import { NetworkTransform, OtherPlayer } from "../components";
import { useClientStore } from "../store";

const lastLocationUpdates = useClientStore.getState().lastLocationUpdates;

export function setLocationUpdateTime(
  players: Query<[OtherPlayer, Mut<NetworkTransform>]>
) {
  for (const [player, network] of players) {
    network.lastUpdate = lastLocationUpdates.get(player.id) ?? 0;
  }
}
