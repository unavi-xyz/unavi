import { editNode, SyncedNode } from "@unavi/engine";
import { useRef, useState } from "react";
import { Euler, Quaternion } from "three";

import { DeepReadonly } from "@/src/play/utils/types";

import NumberInput from "./NumberInput";

const euler = new Euler();
const quat = new Quaternion();

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function Rotation({ node }: Props) {
  const rawX = node.rotation[0] ?? 0;
  const rawY = node.rotation[1] ?? 0;
  const rawZ = node.rotation[2] ?? 0;
  const rawW = node.rotation[3] ?? 0;

  const [uiX, setX] = useState(rawX);
  const [uiY, setY] = useState(rawY);
  const [uiZ, setZ] = useState(rawZ);
  const [uiW, setW] = useState(rawW);

  const [usingUI, setUsingUI] = useState(false);
  const usingUITimout = useRef<NodeJS.Timeout | null>(null);

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
        disabled={node.locked}
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

          editNode(node.id, {
            rotation: [quat.x, quat.y, quat.z, quat.w],
          });
        }}
      />
      <NumberInput
        value={y}
        label="Y"
        placeholder="Y"
        disabled={node.locked}
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

          editNode(node.id, {
            rotation: [quat.x, quat.y, quat.z, quat.w],
          });
        }}
      />
      <NumberInput
        value={z}
        label="Z"
        placeholder="Z"
        disabled={node.locked}
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

          editNode(node.id, {
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
