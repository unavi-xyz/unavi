import { Time } from "lattice-engine/core";
import { PlayerBody } from "lattice-engine/player";
import { Mut, Query, Res } from "thyseus";

import { useClientStore } from "../clientStore";
import { OtherPlayer } from "../components";

export function setPlayersAirTime(
  time: Res<Time>,
  players: Query<[OtherPlayer, Mut<PlayerBody>]>
) {
  for (const [player, body] of players) {
    const playerData = useClientStore.getState().playerData.get(player.id);
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
