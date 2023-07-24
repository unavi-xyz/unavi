import { Quat, Vec3 } from "gl-matrix/dist/esm";
import { Time } from "lattice-engine/core";
import { Transform } from "lattice-engine/scene";
import { Mut, Query, Res } from "thyseus";

import { NetworkTransform } from "../components";
import { NETWORK_UPDATE_HZ } from "../constants";

const vec3 = new Vec3();
const quat = new Quat();

const LERP_HZ = NETWORK_UPDATE_HZ / 2;

export function lerpTransforms(
  time: Res<Time>,
  transforms: Query<[Mut<Transform>, NetworkTransform]>
) {
  for (const [transform, network] of transforms) {
    const timeSinceLast = time.mainTime - network.lastUpdate;
    const percentThroughInterval = timeSinceLast / (1000 / LERP_HZ);
    const K = Math.min(1, percentThroughInterval);

    Vec3.lerp(
      vec3,
      transform.translation.array,
      network.transform.translation.array,
      K
    );
    transform.translation.set(vec3.x, vec3.y, vec3.z);

    Quat.slerp(
      quat,
      transform.rotation.array,
      network.transform.rotation.array,
      K
    );
    transform.rotation.set(quat.x, quat.y, quat.z, quat.w);

    Vec3.lerp(vec3, transform.scale.array, network.transform.scale.array, K);
    transform.scale.set(vec3.x, vec3.y, vec3.z);
  }
}
