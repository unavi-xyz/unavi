import { Quat, Vec3 } from "gl-matrix/dist/esm";
import { Time } from "houseki/core";
import { Transform } from "houseki/scene";
import { Mut, Query, Res } from "thyseus";

import { NetworkTransform } from "../components";
import { NETWORK_UPDATE_HZ } from "../constants";

const vec3 = new Vec3();
const vec3b = new Vec3();
const quat = new Quat();
const quatb = new Quat();

const LERP_HZ = NETWORK_UPDATE_HZ / 2;

export function lerpTransforms(
  time: Res<Time>,
  transforms: Query<[Mut<Transform>, NetworkTransform]>
) {
  for (const [transform, network] of transforms) {
    const timeSinceLast = time.mainTime - network.lastUpdate;
    const percentThroughInterval = timeSinceLast / (1000 / LERP_HZ);
    const K = Math.min(1, percentThroughInterval);

    vec3.x = transform.translation.x;
    vec3.y = transform.translation.y;
    vec3.z = transform.translation.z;

    vec3b.x = network.transform.translation.x;
    vec3b.y = network.transform.translation.y;
    vec3b.z = network.transform.translation.z;

    Vec3.lerp(vec3, vec3, vec3b, K);
    transform.translation.set(vec3.x, vec3.y, vec3.z);

    quat.x = transform.rotation.x;
    quat.y = transform.rotation.y;
    quat.z = transform.rotation.z;
    quat.w = transform.rotation.w;

    quatb.x = network.transform.rotation.x;
    quatb.y = network.transform.rotation.y;
    quatb.z = network.transform.rotation.z;
    quatb.w = network.transform.rotation.w;

    Quat.slerp(quat, quat, quatb, K);
    transform.rotation.set(quat.x, quat.y, quat.z, quat.w);

    vec3.x = transform.scale.x;
    vec3.y = transform.scale.y;
    vec3.z = transform.scale.z;

    vec3b.x = network.transform.scale.x;
    vec3b.y = network.transform.scale.y;
    vec3b.z = network.transform.scale.z;

    Vec3.lerp(vec3, vec3, vec3b, K);
    transform.scale.set(vec3.x, vec3.y, vec3.z);
  }
}
