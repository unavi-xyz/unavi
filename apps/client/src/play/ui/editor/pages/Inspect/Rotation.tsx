import { Euler, Quaternion } from "three";

import { editNode } from "@/src/play/actions/editNode";

import { useTreeValue } from "../../hooks/useTreeValue";
import { useTreeValueKey } from "../../hooks/useTreeValueKey";
import NumberInput from "./NumberInput";

const euler = new Euler();
const quat = new Quaternion();

interface Props {
  id: bigint;
}

export default function Rotation({ id }: Props) {
  const name = useTreeValue(id, "name");
  const locked = useTreeValue(id, "locked");
  const rawX = useTreeValueKey(id, "rotation", 0);
  const rawY = useTreeValueKey(id, "rotation", 1);
  const rawZ = useTreeValueKey(id, "rotation", 2);
  const rawW = useTreeValueKey(id, "rotation", 3);

  if (
    !name ||
    rawX === undefined ||
    rawY === undefined ||
    rawZ === undefined ||
    rawW === undefined
  ) {
    return null;
  }

  euler.setFromQuaternion(quat.set(rawX, rawY, rawZ, rawW));

  const x = round(toDegrees(euler.x));
  const y = round(toDegrees(euler.y));
  const z = round(toDegrees(euler.z));

  return (
    <div className="flex items-center space-x-1">
      <div className="w-20 shrink-0 font-bold text-neutral-400">Rotation</div>

      <NumberInput
        value={x}
        label="X"
        placeholder="X"
        disabled={locked}
        sensitivity={360}
        onValueChange={(val) => {
          quat.setFromEuler(
            euler.set(toRadians(val), toRadians(y), toRadians(z))
          );

          editNode({
            rotation: [quat.x, quat.y, quat.z, quat.w],
            target: name,
          });
        }}
      />
      <NumberInput
        value={y}
        label="Y"
        placeholder="Y"
        disabled={locked}
        sensitivity={360}
        onValueChange={(val) => {
          quat.setFromEuler(
            euler.set(toRadians(x), toRadians(val), toRadians(z))
          );

          editNode({
            rotation: [quat.x, quat.y, quat.z, quat.w],
            target: name,
          });
        }}
      />
      <NumberInput
        value={z}
        label="Z"
        placeholder="Z"
        disabled={locked}
        sensitivity={360}
        onValueChange={(val) => {
          quat.setFromEuler(
            euler.set(toRadians(x), toRadians(y), toRadians(val))
          );

          editNode({
            rotation: [quat.x, quat.y, quat.z, quat.w],
            target: name,
          });
        }}
      />
    </div>
  );
}

const PRECISION = 1000;

function round(num: number) {
  return Math.round(num * PRECISION) / PRECISION;
}

function toRadians(degrees: number) {
  return (degrees * Math.PI) / 180;
}

function toDegrees(radians: number) {
  return (radians * 180) / Math.PI;
}
