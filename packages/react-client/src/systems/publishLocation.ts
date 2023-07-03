import { Time } from "lattice-engine/core";
import { PlayerAvatar, PlayerBody } from "lattice-engine/player";
import { Parent, Transform } from "lattice-engine/scene";
import { Entity, Query, Res, struct, SystemRes, With } from "thyseus";

import { NETWORK_UPDATE_HZ } from "../constants";
import { useClientStore } from "../store";
import { serializeLocation } from "../utils/serializeLocation";

@struct
class LocalRes {
  @struct.f32 declare lastPublish: number;
}

export function publishLocation(
  time: Res<Time>,
  localRes: SystemRes<LocalRes>,
  bodies: Query<[Entity, Transform], With<PlayerBody>>,
  avatars: Query<[Transform, Parent], With<PlayerAvatar>>
) {
  const now = time.fixedTime;
  if (now - localRes.lastPublish < 1000 / NETWORK_UPDATE_HZ) return;

  localRes.lastPublish = now;

  const playerId = useClientStore.getState().playerId;
  if (playerId === null) return;

  for (const [bodyEnt, bodyTr] of bodies) {
    for (const [avatarTr, parent] of avatars) {
      if (parent.id !== bodyEnt.id) continue;

      // TODO: Remove hardcoded player height
      // Define player height within PlayerBody component or something
      const adjustedHeight = bodyTr.translation.y - 0.8;

      const buffer = serializeLocation(
        playerId,
        bodyTr.translation.x,
        adjustedHeight,
        bodyTr.translation.z,
        avatarTr.rotation.x,
        avatarTr.rotation.y,
        avatarTr.rotation.z,
        avatarTr.rotation.w
      );

      useClientStore.getState().sendWebRTC(buffer);
    }
  }
}
