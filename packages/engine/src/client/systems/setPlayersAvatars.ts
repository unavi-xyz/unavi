import { PlayerAvatar } from "lattice-engine/player";
import { Parent } from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Entity, Mut, Query, With } from "thyseus";

import { useClientStore } from "../clientStore";
import { OtherPlayer } from "../components";

export function setPlayersAvatar(
  players: Query<[Entity, OtherPlayer]>,
  avatars: Query<[Parent, Mut<Vrm>], With<PlayerAvatar>>
) {
  for (const [parent, vrm] of avatars) {
    for (const [entity, player] of players) {
      if (entity.id !== parent.id) continue;

      const playerData = useClientStore.getState().playerData.get(player.id);
      if (!playerData) {
        vrm.uri = useClientStore.getState().defaultAvatar;
        continue;
      }

      if (playerData.avatar) {
        vrm.uri = playerData.avatar;
      } else {
        vrm.uri = useClientStore.getState().defaultAvatar;
      }
    }
  }
}
