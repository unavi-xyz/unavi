import { Quat as glQuat, Vec3 as glVec3 } from "gl-matrix/dist/esm";
import { Time, Vec3, Vec4 } from "lattice-engine/core";
import { Transform } from "lattice-engine/scene";
import { Mut, Query, Res } from "thyseus";

import { NetworkTransform } from "../components";
import { NETWORK_UPDATE_HZ } from "../constants";

const LERP_HZ = NETWORK_UPDATE_HZ / 2;

export function lerpTransforms(
  time: Res<Time>,
  transforms: Query<[Mut<Transform>, NetworkTransform]>
) {
  for (const [transform, network] of transforms) {
    const timeSinceLast = time.mainTime - network.lastUpdate;
    const percentThroughInterval = timeSinceLast / (1000 / LERP_HZ);
    const K = Math.min(1, percentThroughInterval);

    lerp(transform.translation, network.transform.translation, K);
    slerp(transform.rotation, network.transform.rotation, K);
    lerp(transform.scale, network.transform.scale, K);
  }
}

const quat = new glQuat();
const quatb = new glQuat();

function slerp(current: Vec4, target: Vec4, K: number) {
  glQuat.set(quat, current.x, current.y, current.z, current.w);
  glQuat.set(quatb, target.x, target.y, target.z, target.w);

  glQuat.slerp(quat, quat, quatb, K);

  current.fromObject(quat);
}

const vec3 = new glVec3();
const vec3b = new glVec3();

function lerp(current: Vec3, target: Vec3, K: number) {
  glVec3.set(vec3, current.x, current.y, current.z);
  glVec3.set(vec3b, target.x, target.y, target.z);

  glVec3.lerp(vec3, vec3, vec3b, K);

  current.fromObject(vec3);
}
