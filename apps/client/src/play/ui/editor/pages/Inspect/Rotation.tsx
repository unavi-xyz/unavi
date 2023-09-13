import { useRef, useState } from "react";
import { Euler, Quaternion } from "three";

import { editNode } from "@/src/play/actions/editNode";

import { useTreeValue } from "../../hooks/useTreeValue";
import { useTreeValueIndex } from "../../hooks/useTreeValueIndex";
import NumberInput from "./NumberInput";

const euler = new Euler();
const quat = new Quaternion();

interface Props {
  entityId: bigint;
}

export default function Rotation({ entityId }: Props) {
  const id = useTreeValue(entityId, "id");
  const name = useTreeValue(entityId, "name");
  const locked = useTreeValue(entityId, "locked");

  const rawX = useTreeValueIndex(entityId, "rotation", 0);
  const rawY = useTreeValueIndex(entityId, "rotation", 1);
  const rawZ = useTreeValueIndex(entityId, "rotation", 2);
  const rawW = useTreeValueIndex(entityId, "rotation", 3);

  const [uiX, setX] = useState(rawX ?? 0);
  const [uiY, setY] = useState(rawY ?? 0);
  const [uiZ, setZ] = useState(rawZ ?? 0);
  const [uiW, setW] = useState(rawW ?? 0);

  const [usingUI, setUsingUI] = useState(false);
  const usingUITimout = useRef<NodeJS.Timeout | null>(null);

  if (!id || !name) {
    return null;
  }

  function usedUI() {
    setUsingUI(true);
    setX(rawX ?? 0);
    setY(rawY ?? 0);
    setZ(rawZ ?? 0);
    setW(rawW ?? 0);

    if (usingUITimout.current) {
      clearTimeout(usingUITimout.current);
    }

    usingUITimout.current = setTimeout(() => {
      setUsingUI(false);
    }, 100);
  }

  if (!id || !name) {
    return null;
  }

  const qx = usingUI ? uiX : rawX ?? 0;
  const qy = usingUI ? uiY : rawY ?? 0;
  const qz = usingUI ? uiZ : rawZ ?? 0;
  const qw = usingUI ? uiW : rawW ?? 0;

  euler.setFromQuaternion(quat.set(qx, qy, qz, qw));

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

          usedUI();
          setX(quat.x);
          setY(quat.y);
          setZ(quat.z);
          setW(quat.w);

          editNode(id, {
            rotation: [quat.x, quat.y, quat.z, quat.w],
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

          usedUI();
          setX(quat.x);
          setY(quat.y);
          setZ(quat.z);
          setW(quat.w);

          editNode(id, {
            rotation: [quat.x, quat.y, quat.z, quat.w],
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

          usedUI();
          setX(quat.x);
          setY(quat.y);
          setZ(quat.z);
          setW(quat.w);

          editNode(id, {
            rotation: [quat.x, quat.y, quat.z, quat.w],
          });
        }}
      />
    </div>
  );
}

function round(num: number, precision = 1000) {
  return Math.round(num * precision) / precision;
}

function toRadians(degrees: number) {
  return (degrees * Math.PI) / 180;
}

function toDegrees(radians: number) {
  return (radians * 180) / Math.PI;
}
