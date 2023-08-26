import { Mut, Query } from "thyseus";

import { NetworkTransform, OtherPlayer } from "../components";
import { connectionStore } from "./connectToHost";

export function setLocationUpdateTime(
  players: Query<[OtherPlayer, Mut<NetworkTransform>]>
) {
  const lastLocationUpdates = connectionStore.get(
    connectionStore.lastLocationUpdates
  );

  for (const [player, network] of players) {
    network.lastUpdate = lastLocationUpdates.get(player.id) ?? 0;
  }
}
