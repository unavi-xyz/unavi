import { Vec3 } from "gl-matrix/dist/esm";
import { Time } from "lattice-engine/core";
import { Velocity } from "lattice-engine/physics";
import { Transform } from "lattice-engine/scene";
import { Mut, Query, Res, With } from "thyseus";

import { OtherPlayer, PrevTranslation } from "../components";

const vec3 = new Vec3();

export function calcPlayerVelocity(
  time: Res<Time>,
  players: Query<
    [Transform, Mut<PrevTranslation>, Mut<Velocity>],
    With<OtherPlayer>
  >
) {
  const K = 1 - Math.pow(10e-8, time.mainDelta);

  for (const [transform, prev, velocity] of players) {
    const deltaX = transform.translation.x - prev.x;
    const deltaY = transform.translation.y - prev.y;
    const deltaZ = transform.translation.z - prev.z;

    prev.x = transform.translation.x;
    prev.y = transform.translation.y;
    prev.z = transform.translation.z;

    Vec3.lerp(
      vec3,
      [velocity.x, velocity.y, velocity.z],
      [
        deltaX / time.mainDelta,
        deltaY / time.mainDelta,
        deltaZ / time.mainDelta,
      ],
      K
    );

    velocity.x = vec3.x;
    velocity.y = vec3.y;
    velocity.z = vec3.z;
  }
}
