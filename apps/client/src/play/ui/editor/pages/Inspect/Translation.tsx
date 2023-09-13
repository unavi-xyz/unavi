import { useRef, useState } from "react";

import { editNode } from "@/src/play/actions/editNode";

import { useTreeValue } from "../../hooks/useTreeValue";
import { useTreeValueIndex } from "../../hooks/useTreeValueIndex";
import NumberInput from "./NumberInput";

interface Props {
  entityId: bigint;
}

export default function Translation({ entityId }: Props) {
  const id = useTreeValue(entityId, "id");
  const name = useTreeValue(entityId, "name");
  const locked = useTreeValue(entityId, "locked");

  const rawX = useTreeValueIndex(entityId, "translation", 0);
  const rawY = useTreeValueIndex(entityId, "translation", 1);
  const rawZ = useTreeValueIndex(entityId, "translation", 2);

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
      <div className="w-20 shrink-0 font-bold text-neutral-400">Position</div>

      <NumberInput
        value={roundedX}
        label="X"
        placeholder="X"
        disabled={locked}
        onValueChange={(val) => {
          usedUI();
          setX(val);
          editNode(id, {
            translation: [val, y, z],
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
            translation: [x, val, z],
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
            translation: [x, y, val],
          });
        }}
      />
    </div>
  );
}

function round(num: number, precision = 1000) {
  return Math.round(num * precision) / precision;
}
