import { Euler, Quaternion } from "three";

import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useTreeArrayValue } from "../../hooks/useTreeArrayValue";
import { useTreeValue } from "../../hooks/useTreeValue";

const STEP = 1;

const euler = new Euler();
const quat = new Quaternion();

interface Props {
  id: bigint;
}

export default function Rotation({ id }: Props) {
  const name = useTreeValue(id, "name");
  const rawX = useTreeArrayValue(id, "rotation", 0);
  const rawY = useTreeArrayValue(id, "rotation", 1);
  const rawZ = useTreeArrayValue(id, "rotation", 2);
  const rawW = useTreeArrayValue(id, "rotation", 3);

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
    <div>
      <div className="font-bold text-neutral-400">Rotation</div>

      <div className="flex space-x-2">
        <TextFieldDark
          value={x}
          type="number"
          step={STEP}
          placeholder="X"
          onChange={(e) => {
            quat.setFromEuler(
              euler.set(
                toRadians(Number(e.target.value)),
                toRadians(y),
                toRadians(z)
              )
            );

            editNode({
              rotation: [quat.x, quat.y, quat.z, quat.w],
              target: name,
            });
          }}
        />
        <TextFieldDark
          value={y}
          type="number"
          step={STEP}
          placeholder="Y"
          onChange={(e) => {
            quat.setFromEuler(
              euler.set(
                toRadians(x),
                toRadians(Number(e.target.value)),
                toRadians(z)
              )
            );

            editNode({
              rotation: [quat.x, quat.y, quat.z, quat.w],
              target: name,
            });
          }}
        />
        <TextFieldDark
          value={z}
          type="number"
          step={STEP}
          placeholder="Z"
          onChange={(e) => {
            quat.setFromEuler(
              euler.set(
                toRadians(x),
                toRadians(y),
                toRadians(Number(e.target.value))
              )
            );

            editNode({
              rotation: [quat.x, quat.y, quat.z, quat.w],
              target: name,
            });
          }}
        />
      </div>
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
