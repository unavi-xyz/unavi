import { Box } from "@wired-labs/engine";

import NumberInput from "../../../ui/base/NumberInput";
import { setGeometry } from "../../actions/SetGeometry";
import { useEditorStore } from "../../store";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  entityId: string;
}

export default function BoxComponent({ entityId }: Props) {
  const width = useEditorStore((state) => {
    const entity = state.scene.entities[entityId] as Box;
    return entity.width;
  });
  const height = useEditorStore((state) => {
    const entity = state.scene.entities[entityId] as Box;
    return entity.height;
  });
  const depth = useEditorStore((state) => {
    const entity = state.scene.entities[entityId] as Box;
    return entity.depth;
  });

  return (
    <ComponentMenu title="Geometry">
      <MenuRows titles={["Width", "Height", "Depth"]}>
        {[width, height, depth].map((value, i) => {
          const property = i === 0 ? "width" : i === 1 ? "height" : "depth";
          const name = ["Width", "Height", "Depth"][i];

          return (
            <NumberInput
              key={name}
              name={name}
              value={value}
              step={0.1}
              onChange={(e) => {
                // @ts-ignore
                const value: string | null = e.target.value || null;
                if (value === null) return;

                const num = parseFloat(value);
                const rounded = Math.round(num * 1000) / 1000;

                const { scene } = useEditorStore.getState();
                const entity = scene.entities[entityId] as Box;
                entity[property] = rounded;

                setGeometry(entity);
              }}
            />
          );
        })}
      </MenuRows>
    </ComponentMenu>
  );
}
