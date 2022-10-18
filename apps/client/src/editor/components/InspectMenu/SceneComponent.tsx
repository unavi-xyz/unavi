import { Triplet } from "@wired-labs/engine";

import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import NumberInput from "../ui/NumberInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

export default function SceneComponent() {
  const spawn$ = useEditorStore((state) => state.engine?.scene.spawn$);
  const spawn = useSubscribeValue(spawn$);

  return (
    <ComponentMenu removeable={false}>
      {spawn && (
        <MenuRows titles={["Spawn"]}>
          <div className="grid grid-cols-3 gap-3">
            {spawn.map((value, i) => {
              const letter = ["X", "Y", "Z"][i];
              const name = `position-${letter}`;

              const rounded = Math.round(value * 1000) / 1000;

              return (
                <div key={name} className="flex space-x-2">
                  <label htmlFor={name}>{letter}</label>
                  <NumberInput
                    name={name}
                    value={rounded}
                    step={0.1}
                    onChange={(e) => {
                      // @ts-ignore
                      const value: string | null = e.target.value || null;
                      if (value === null) return;

                      const num = parseFloat(value);
                      const rounded = Math.round(num * 1000) / 1000;

                      const newPosition: Triplet = [...spawn];
                      newPosition[i] = rounded;

                      spawn$?.next(newPosition);
                    }}
                  />
                </div>
              );
            })}
          </div>
        </MenuRows>
      )}
    </ComponentMenu>
  );
}
