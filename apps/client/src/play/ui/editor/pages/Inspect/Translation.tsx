import { editNode } from "@/src/play/actions/editNode";

import { useTreeValue } from "../../hooks/useTreeValue";
import { useTreeValueKey } from "../../hooks/useTreeValueKey";
import NumberInput from "./NumberInput";

interface Props {
  id: bigint;
}

export default function Translation({ id }: Props) {
  const name = useTreeValue(id, "name");
  const locked = useTreeValue(id, "locked");
  const rawX = useTreeValueKey(id, "translation", 0);
  const rawY = useTreeValueKey(id, "translation", 1);
  const rawZ = useTreeValueKey(id, "translation", 2);

  if (!name || rawX === undefined || rawY === undefined || rawZ === undefined) {
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
          editNode({
            target: name,
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
          editNode({
            target: name,
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
          editNode({
            target: name,
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
