import { editNode, SyncedNode } from "@unavi/engine";
import { useRef, useState } from "react";

import { DeepReadonly } from "@/src/play/utils/types";

import NumberInput from "./NumberInput";

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function Scale({ node }: Props) {
  const rawX = node.scale[0] ?? 0;
  const rawY = node.scale[1] ?? 0;
  const rawZ = node.scale[2] ?? 0;

  const [uiX, setX] = useState(rawX);
  const [uiY, setY] = useState(rawY);
  const [uiZ, setZ] = useState(rawZ);

  const [usingUI, setUsingUI] = useState(false);
  const usingUITimout = useRef<NodeJS.Timeout | null>(null);

  function usedUI() {
    setUsingUI(true);
    setX(rawX ?? 0);
    setY(rawY ?? 0);
    setZ(rawZ ?? 0);

    if (usingUITimout.current) {
      clearTimeout(usingUITimout.current);
    }

    usingUITimout.current = setTimeout(() => {
      setUsingUI(false);
    }, 100);
  }

  const x = usingUI ? uiX : rawX ?? 0;
  const y = usingUI ? uiY : rawY ?? 0;
  const z = usingUI ? uiZ : rawZ ?? 0;

  const roundedX = round(x);
  const roundedY = round(y);
  const roundedZ = round(z);

  return (
    <div className="flex items-center space-x-1">
      <div className="w-20 shrink-0 font-bold text-neutral-400">Scale</div>

      <NumberInput
        value={roundedX}
        label="X"
        placeholder="X"
        disabled={node.locked}
        onValueChange={(val) => {
          usedUI();
          setX(val);
          editNode(node.id, {
            scale: [val, y, z],
          });
        }}
      />
      <NumberInput
        value={roundedY}
        label="Y"
        placeholder="Y"
        disabled={node.locked}
        onValueChange={(val) => {
          usedUI();
          setY(val);
          editNode(node.id, {
            scale: [x, val, z],
          });
        }}
      />
      <NumberInput
        value={roundedZ}
        label="Z"
        placeholder="Z"
        disabled={node.locked}
        onValueChange={(val) => {
          usedUI();
          setZ(val);
          editNode(node.id, {
            scale: [x, y, val],
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
