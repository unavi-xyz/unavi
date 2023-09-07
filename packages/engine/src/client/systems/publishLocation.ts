import { SetPlayerData } from "@wired-protocol/types";
import { Time } from "lattice-engine/core";
import { PlayerBody, PlayerCamera } from "lattice-engine/player";
import { Transform } from "lattice-engine/scene";
import { Entity, f32, Query, Res, struct, SystemRes } from "thyseus";

import { useClientStore } from "../clientStore";
import { NETWORK_UPDATE_HZ } from "../constants";
import { serializeLocation } from "../utils/serializeLocation";

const FALL_THRESHOLD_SECONDS = 0.35;

@struct
class LocalRes {
  lastPublish: f32 = 0;
  isFalling: boolean = false;
}

export function publishLocation(
  time: Res<Time>,
  localRes: SystemRes<LocalRes>,
  bodies: Query<[Entity, Transform, PlayerBody]>,
  cameras: Query<PlayerCamera>
) {
  const now = time.fixedTime;
  if (now - localRes.lastPublish < 1000 / NETWORK_UPDATE_HZ) return;

  localRes.lastPublish = now;

  const playerId = useClientStore.getState().playerId;
  if (playerId === null) return;

  for (const [bodyEnt, transform, body] of bodies) {
    for (const camera of cameras) {
      // Ensure we are only grabbing the user's player body
      // Which is the one with a PlayerCamera
      if (camera.bodyId !== bodyEnt.id) continue;

      // Publish falling state
      const longAirtime = body.airTime > FALL_THRESHOLD_SECONDS;
      const isJumping = body.jumpTime > 0 && body.airTime !== 0;
      const isFalling = longAirtime || isJumping;

      if (isFalling !== localRes.isFalling) {
        localRes.isFalling = isFalling;

        const setPlayerData = SetPlayerData.create({
          data: {
            falling: JSON.stringify(isFalling),
          },
        });

        useClientStore
          .getState()
          .sendWebSockets({ oneofKind: "setPlayerData", setPlayerData });
      }

      // TODO: Remove hardcoded player height
      // Define player height within PlayerBody component or something
      const adjustedHeight = transform.translation.y - 0.8;

      const buffer = serializeLocation(
        playerId,
        transform.translation.x,
        adjustedHeight,
        transform.translation.z,
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
        transform.rotation.w
      );

      useClientStore.getState().sendWebRTC(buffer);
    }
  }
}
