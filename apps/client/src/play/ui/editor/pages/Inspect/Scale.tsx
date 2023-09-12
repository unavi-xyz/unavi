import { editNode } from "@/src/play/actions/editNode";

import { useTreeValue } from "../../hooks/useTreeValue";
import { useTreeValueIndex } from "../../hooks/useTreeValueIndex";
import NumberInput from "./NumberInput";

interface Props {
  id: bigint;
}

export default function Scale({ id }: Props) {
  const name = useTreeValue(id, "name");
  const locked = useTreeValue(id, "locked");
  const rawX = useTreeValueIndex(id, "scale", 0);
  const rawY = useTreeValueIndex(id, "scale", 1);
  const rawZ = useTreeValueIndex(id, "scale", 2);

  if (!name || rawX === undefined || rawY === undefined || rawZ === undefined) {
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
          editNode({
            scale: [val, y, z],
            target: name,
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
            scale: [x, val, z],
            target: name,
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
            scale: [x, y, val],
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
