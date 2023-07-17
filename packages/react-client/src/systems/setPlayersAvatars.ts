import { PlayerAvatar } from "lattice-engine/player";
import { Parent } from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Entity, Mut, Query, With } from "thyseus";

import { OtherPlayer } from "../components";
import { useClientStore } from "../store";

export function setPlayersAvatar(
  players: Query<[Entity, OtherPlayer]>,
  avatars: Query<[Parent, Mut<Vrm>], With<PlayerAvatar>>
) {
  for (const [parent, vrm] of avatars) {
    for (const [entity, player] of players) {
      if (entity.id !== parent.id) continue;

      const avatar = useClientStore.getState().avatars.get(player.id);

      if (avatar) {
        vrm.uri = avatar;
      } else {
        vrm.uri = useClientStore.getState().defaultAvatar;
      }
    }
  }
}
