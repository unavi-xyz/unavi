import { Time } from "lattice-engine/core";
import { PlayerBody } from "lattice-engine/player";
import { Mut, Query, Res } from "thyseus";

import { OtherPlayer } from "../components";
import { useClientStore } from "../store";

const grounded = useClientStore.getState().grounded;

export function setPlayersAirTime(
  time: Res<Time>,
  players: Query<[OtherPlayer, Mut<PlayerBody>]>
) {
  for (const [player, body] of players) {
    const isGrounded = grounded.get(player.id) ?? true;

    if (isGrounded) {
      body.airTime = 0;
      body.jumpTime = 0;
    } else {
      body.airTime += time.mainDelta;
      body.jumpTime += time.mainDelta;
    }
  }
}
