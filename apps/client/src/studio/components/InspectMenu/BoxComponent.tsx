import { Box } from "@wired-labs/engine";

import NumberInput from "../../../ui/base/NumberInput";
import { setGeometry } from "../../actions/SetGeometry";
import { useStudioStore } from "../../store";
import ComponentMenu from "./ComponentMenu";

interface Props {
  entityId: string;
}

export default function BoxComponent({ entityId }: Props) {
  const width = useStudioStore((state) => {
    const entity = state.tree[entityId] as Box;
    return entity.width;
  });
  const height = useStudioStore((state) => {
    const entity = state.tree[entityId] as Box;
    return entity.height;
  });
  const depth = useStudioStore((state) => {
    const entity = state.tree[entityId] as Box;
    return entity.depth;
  });

  return (
    <ComponentMenu title="Geometry">
      {[width, height, depth].map((value, i) => {
        const property = i === 0 ? "width" : i === 1 ? "height" : "depth";
        const name = ["Width", "Height", "Depth"][i];

        return (
          <div key={name} className="grid grid-cols-2 gap-3">
            <label htmlFor={name}>{name}</label>
            <div className="flex">
              <NumberInput
                name={name}
                value={value}
                step={0.1}
                onChange={(e) => {
                  // @ts-ignore
                  const value: string | null = e.target.value || null;
                  if (value === null) return;

                  const num = parseFloat(value);
                  const rounded = Math.round(num * 1000) / 1000;

                  const { tree } = useStudioStore.getState();
                  const entity = tree[entityId] as Box;
                  entity[property] = rounded;

                  setGeometry(entity);
                }}
              />
            </div>
          </div>
        );
      })}
    </ComponentMenu>
  );
}
