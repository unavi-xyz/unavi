import { useRef, useState } from "react";

import { editNode } from "@/src/play/actions/editNode";

import { useTreeValue } from "../../hooks/useTreeValue";
import { useTreeValueIndex } from "../../hooks/useTreeValueIndex";
import NumberInput from "./NumberInput";

interface Props {
  entityId: bigint;
}

export default function Scale({ entityId }: Props) {
  const id = useTreeValue(entityId, "id");
  const name = useTreeValue(entityId, "name");
  const locked = useTreeValue(entityId, "locked");

  const rawX = useTreeValueIndex(entityId, "scale", 0);
  const rawY = useTreeValueIndex(entityId, "scale", 1);
  const rawZ = useTreeValueIndex(entityId, "scale", 2);

  const [uiX, setX] = useState(rawX ?? 0);
  const [uiY, setY] = useState(rawY ?? 0);
  const [uiZ, setZ] = useState(rawZ ?? 0);

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
        disabled={locked}
        onValueChange={(val) => {
          usedUI();
          setX(val);
          editNode(id, {
            scale: [val, y, z],
          });
        }}
      />
      <NumberInput
        value={roundedY}
        label="Y"
        placeholder="Y"
        disabled={locked}
        onValueChange={(val) => {
          usedUI();
          setY(val);
          editNode(id, {
            scale: [x, val, z],
          });
        }}
      />
      <NumberInput
        value={roundedZ}
        label="Z"
        placeholder="Z"
        disabled={locked}
        onValueChange={(val) => {
          usedUI();
          setZ(val);
          editNode(id, {
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
