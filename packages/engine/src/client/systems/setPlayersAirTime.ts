import { Time } from "lattice-engine/core";
import { PlayerBody } from "lattice-engine/player";
import { Mut, Query, Res } from "thyseus";

import { OtherPlayer } from "../components";
import { connectionStore } from "./connectToHost";

export function setPlayersAirTime(
  time: Res<Time>,
  players: Query<[OtherPlayer, Mut<PlayerBody>]>
) {
  for (const [player, body] of players) {
    const playerData = connectionStore
      .get(connectionStore.playerData)
      ?.get(player.id);
    if (!playerData) continue;

    const isFalling = playerData.falling === "true";

    if (isFalling) {
      body.airTime += time.mainDelta;
      body.jumpTime += time.mainDelta;
    } else {
      body.airTime = 0;
      body.jumpTime = 0;
    }
  }
}
