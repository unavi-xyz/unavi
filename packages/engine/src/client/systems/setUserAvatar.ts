import { Warehouse } from "lattice-engine/core";
import { PlayerAvatar, PlayerBody, PlayerCamera } from "lattice-engine/player";
import { Parent } from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Entity, Mut, Query, Res, With } from "thyseus";

import { useClientStore } from "../clientStore";

export function setUserAvatar(
  warehouse: Res<Mut<Warehouse>>,
  bodies: Query<Entity, With<PlayerBody>>,
  avatars: Query<[Parent, Mut<Vrm>], With<PlayerAvatar>>,
  cameras: Query<PlayerCamera>
) {
  for (const camera of cameras) {
    for (const entity of bodies) {
      if (camera.bodyId !== entity.id) continue;

      for (const [avatarParent, vrm] of avatars) {
        if (avatarParent.id !== entity.id) continue;

        const { avatar, defaultAvatar } = useClientStore.getState();
        const usedAvatar = avatar || defaultAvatar;
        const uri = vrm.uri.read(warehouse) ?? "";
        if (uri === usedAvatar) continue;

        vrm.uri.write(usedAvatar, warehouse);
      }
    }
  }
}
