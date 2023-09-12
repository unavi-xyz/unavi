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

  if (
    !id ||
    !name ||
    rawX === undefined ||
    rawY === undefined ||
    rawZ === undefined
  ) {
    return null;
  }

  const x = round(rawX);
  const y = round(rawY);
  const z = round(rawZ);

  return (
    <div className="flex items-center space-x-1">
      <div className="w-20 shrink-0 font-bold text-neutral-400">Scale</div>

      <NumberInput
        value={x}
        label="X"
        placeholder="X"
        disabled={locked}
        onValueChange={(val) => {
          editNode(id, {
            scale: [val, y, z],
          });
        }}
      />
      <NumberInput
        value={y}
        label="Y"
        placeholder="Y"
        disabled={locked}
        onValueChange={(val) => {
          editNode(id, {
            scale: [x, val, z],
          });
        }}
      />
      <NumberInput
        value={z}
        label="Z"
        placeholder="Z"
        disabled={locked}
        onValueChange={(val) => {
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
