import { PlayerAvatar, PlayerBody, PlayerCamera } from "houseki/player";
import { Parent } from "houseki/scene";
import { Vrm } from "houseki/vrm";
import { Entity, Mut, Query, With } from "thyseus";

import { useClientStore } from "../clientStore";

export function setUserAvatar(
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
        if (vrm.uri === usedAvatar) continue;

        vrm.uri = usedAvatar;
      }
    }
  }
}
