import { Time } from "lattice-engine/core";
import { PlayerBody } from "lattice-engine/player";
import { Mut, Query, Res } from "thyseus";

import { useClientStore } from "../clientStore";
import { OtherPlayer } from "../components";

const falling = useClientStore.getState().falling;

export function setPlayersAirTime(
  time: Res<Time>,
  players: Query<[OtherPlayer, Mut<PlayerBody>]>
) {
  for (const [player, body] of players) {
    const isFalling = falling.get(player.id) ?? false;

    if (isFalling) {
      body.airTime += time.mainDelta;
      body.jumpTime += time.mainDelta;
    } else {
      body.airTime = 0;
      body.jumpTime = 0;
    }
  }
}
