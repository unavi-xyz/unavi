import { Time } from "lattice-engine/core";
import { PlayerAvatar, PlayerBody, PlayerCamera } from "lattice-engine/player";
import { Parent, Transform } from "lattice-engine/scene";
import { Entity, Query, Res, struct, SystemRes, With } from "thyseus";

import { NETWORK_UPDATE_HZ } from "../constants";
import { useClientStore } from "../store";
import { serializeLocation } from "../utils/serializeLocation";

const FALL_THRESHOLD_SECONDS = 0.35;

@struct
class LocalRes {
  @struct.f32 declare lastPublish: number;
  @struct.bool declare isGrounded: boolean;
}

export function publishLocation(
  time: Res<Time>,
  localRes: SystemRes<LocalRes>,
  bodies: Query<[Entity, Transform, PlayerBody]>,
  avatars: Query<[Transform, Parent], With<PlayerAvatar>>,
  cameras: Query<Parent, With<PlayerCamera>>
) {
  const now = time.fixedTime;
  if (now - localRes.lastPublish < 1000 / NETWORK_UPDATE_HZ) return;

  localRes.lastPublish = now;

  const playerId = useClientStore.getState().playerId;
  if (playerId === null) return;

  for (const [bodyEnt, bodyTr, body] of bodies) {
    for (const parent of cameras) {
      // Ensure we are only grabbing the user's player body
      // Which is the one with a PlayerCamera
      if (parent.id !== bodyEnt.id) continue;

      // Publish grounded state
      // TODO: Change this to falling state
      const isFalling = body.airTime > FALL_THRESHOLD_SECONDS;
      const isJumping = body.jumpTime > 0 && body.airTime !== 0;
      const isGrounded = !isFalling && !isJumping;

      if (isGrounded !== localRes.isGrounded) {
        localRes.isGrounded = isGrounded;
        useClientStore.getState().sendWebSockets({
          data: isGrounded,
          id: "xyz.unavi.world.user.grounded",
        });
      }

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
}
