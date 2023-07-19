import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useTreeArrayValue } from "../../hooks/useTreeArrayValue";
import { useTreeValue } from "../../hooks/useTreeValue";

const STEP = 0.1;

interface Props {
  id: bigint;
}

export default function Translation({ id }: Props) {
  const name = useTreeValue(id, "name");
  const rawX = useTreeArrayValue(id, "translation", 0);
  const rawY = useTreeArrayValue(id, "translation", 1);
  const rawZ = useTreeArrayValue(id, "translation", 2);

  if (!name || rawX === undefined || rawY === undefined || rawZ === undefined) {
    return null;
  }

  const x = round(rawX);
  const y = round(rawY);
  const z = round(rawZ);

  return (
    <div>
      <div className="font-bold text-neutral-400">Translation</div>

      <div className="flex space-x-2">
        <TextFieldDark
          value={x}
          type="number"
          step={STEP}
          placeholder="X"
          onChange={(e) => {
            editNode({
              target: name,
              translation: [Number(e.target.value), y, z],
            });
          }}
        />
        <TextFieldDark
          value={y}
          type="number"
          step={STEP}
          placeholder="Y"
          onChange={(e) => {
            editNode({
              target: name,
              translation: [x, Number(e.target.value), z],
            });
          }}
        />
        <TextFieldDark
          value={z}
          type="number"
          step={STEP}
          placeholder="Z"
          onChange={(e) => {
            editNode({
              target: name,
              translation: [x, y, Number(e.target.value)],
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
