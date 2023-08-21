import { Warehouse } from "lattice-engine/core";
import { PlayerAvatar } from "lattice-engine/player";
import { Parent } from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Entity, Mut, Query, Res, With } from "thyseus";

import { useClientStore } from "../clientStore";
import { OtherPlayer } from "../components";

export function setPlayersAvatar(
  warehouse: Res<Mut<Warehouse>>,
  players: Query<[Entity, OtherPlayer]>,
  avatars: Query<[Parent, Mut<Vrm>], With<PlayerAvatar>>
) {
  for (const [parent, vrm] of avatars) {
    for (const [entity, player] of players) {
      if (entity.id !== parent.id) continue;

      const playerData = useClientStore.getState().playerData.get(player.id);
      if (!playerData) {
        vrm.uri.write(useClientStore.getState().defaultAvatar, warehouse);
        continue;
      }

      if (playerData.avatar) {
        vrm.uri.write(playerData.avatar, warehouse);
      } else {
        vrm.uri.write(useClientStore.getState().defaultAvatar, warehouse);
      }
    }
  }
}
