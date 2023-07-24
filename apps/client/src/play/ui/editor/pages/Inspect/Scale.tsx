import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useTreeArrayValue } from "../../hooks/useTreeArrayValue";
import { useTreeValue } from "../../hooks/useTreeValue";

const STEP = 0.01;

interface Props {
  id: bigint;
}

export default function Scale({ id }: Props) {
  const name = useTreeValue(id, "name");
  const locked = useTreeValue(id, "locked");
  const rawX = useTreeArrayValue(id, "scale", 0);
  const rawY = useTreeArrayValue(id, "scale", 1);
  const rawZ = useTreeArrayValue(id, "scale", 2);

  if (!name || rawX === undefined || rawY === undefined || rawZ === undefined) {
    return null;
  }

  const x = round(rawX);
  const y = round(rawY);
  const z = round(rawZ);

  return (
    <div>
      <div className="font-bold text-neutral-400">Scale</div>

      <div className="flex space-x-2">
        <TextFieldDark
          value={x}
          type="number"
          step={STEP}
          placeholder="X"
          disabled={locked}
          onChange={(e) => {
            editNode({
              scale: [Number(e.target.value), y, z],
              target: name,
            });
          }}
        />
        <TextFieldDark
          value={y}
          type="number"
          step={STEP}
          placeholder="Y"
          disabled={locked}
          onChange={(e) => {
            editNode({
              scale: [x, Number(e.target.value), z],
              target: name,
            });
          }}
        />
        <TextFieldDark
          value={z}
          type="number"
          step={STEP}
          placeholder="Z"
          disabled={locked}
          onChange={(e) => {
            editNode({
              scale: [x, y, Number(e.target.value)],
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
