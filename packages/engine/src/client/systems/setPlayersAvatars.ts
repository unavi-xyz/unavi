import { getDefaultStore } from "jotai";
import { Warehouse } from "lattice-engine/core";
import { PlayerAvatar } from "lattice-engine/player";
import { Parent } from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Entity, Mut, Query, Res, With } from "thyseus";

import { OtherPlayer } from "../components";
import { connectionStore } from "./connectToHost";
import { defaultAvatarAtom } from "./spawnPlayers";

export function setPlayersAvatar(
  warehouse: Res<Mut<Warehouse>>,
  players: Query<[Entity, OtherPlayer]>,
  avatars: Query<[Parent, Mut<Vrm>], With<PlayerAvatar>>
) {
  for (const [parent, vrm] of avatars) {
    for (const [entity, player] of players) {
      if (entity.id !== parent.id) continue;

      const playerData = connectionStore
        .get(connectionStore.playerData)
        ?.get(player.id);

      const defaultAvatar = getDefaultStore().get(defaultAvatarAtom);

      if (!playerData) {
        vrm.uri.write(defaultAvatar, warehouse);
        continue;
      }

      if (playerData.avatar) {
        vrm.uri.write(playerData.avatar, warehouse);
      } else {
        vrm.uri.write(defaultAvatar, warehouse);
      }
    }
  }
}
