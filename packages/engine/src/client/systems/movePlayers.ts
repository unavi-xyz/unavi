import { Mut, Query } from "thyseus";

import { NetworkTransform, OtherPlayer } from "../components";
import { connectionStore } from "./connectToHost";

export function movePlayers(
  players: Query<[OtherPlayer, Mut<NetworkTransform>]>
) {
  const locations = connectionStore.get(connectionStore.locations);

  for (const [player, network] of players) {
    const location = locations.get(player.id);
    if (!location) continue;

    const posX = location[0] ?? 0;
    const posY = location[1] ?? 0;
    const posZ = location[2] ?? 0;

    const rotX = location[3] ?? 0;
    const rotY = location[4] ?? 0;
    const rotZ = location[5] ?? 0;
    const rotW = location[6] ?? 0;

    network.transform.translation.set(posX, posY, posZ);
    network.transform.rotation.set(rotX, rotY, rotZ, rotW);
  }
}
