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
      <div className="w-20 shrink-0 font-bold text-neutral-400">Position</div>

      <NumberInput
        value={x}
        label="X"
        placeholder="X"
        disabled={locked}
        onValueChange={(val) => {
          editNode(id, {
            translation: [val, y, z],
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
            translation: [x, val, z],
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
            translation: [x, y, val],
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
