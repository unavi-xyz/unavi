import { Vec3, Vec4 } from "engine";

export function quaternionToEuler([x, y, z, w]: Vec4): Vec3 {
  // Rotate Matrix4
  const te = [];

  const x2 = x + x,
    y2 = y + y,
    z2 = z + z;
  const xx = x * x2,
    xy = x * y2,
    xz = x * z2;
  const yy = y * y2,
    yz = y * z2,
    zz = z * z2;
  const wx = w * x2,
    wy = w * y2,
    wz = w * z2;

  const sx = 1;
  const sy = 1;
  const sz = 1;

  const px = 0;
  const py = 0;
  const pz = 0;

  te[0] = (1 - (yy + zz)) * sx;
  te[1] = (xy + wz) * sx;
  te[2] = (xz - wy) * sx;
  te[3] = 0;

  te[4] = (xy - wz) * sy;
  te[5] = (1 - (xx + zz)) * sy;
  te[6] = (yz + wx) * sy;
  te[7] = 0;

  te[8] = (xz + wy) * sz;
  te[9] = (yz - wx) * sz;
  te[10] = (1 - (xx + yy)) * sz;
  te[11] = 0;

  te[12] = px;
  te[13] = py;
  te[14] = pz;
  te[15] = 1;

  // To Euler
  const m11 = te[0];
  const m12 = te[4];
  const m13 = te[8];

  // const m21 = te[1];
  const m22 = te[5];
  const m23 = te[9];

  // const m31 = te[2];
  const m32 = te[6];
  const m33 = te[10];

  const eulerY = Math.asin(clamp(m13, -1, 1));

  let eulerX: number;
  let eulerZ: number;

  if (Math.abs(m13) < 0.9999999) {
    eulerX = Math.atan2(-m23, m33);
    eulerZ = Math.atan2(-m12, m11);
  } else {
    eulerX = Math.atan2(m32, m22);
    eulerZ = 0;
  }

  return [eulerX, eulerY, eulerZ];
}

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}
