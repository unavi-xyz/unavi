import { useRef } from "react";

import { Entity } from "@wired-xr/engine";

import { round } from "../../../../helpers/utils/round";
import ColorInput from "../../../base/ColorInput";
import MenuRow from "../MenuRow";
import NumberInput from "../NumberInput";

interface Props {
  selected: Entity<"PointLight">;
  handleChange: (key: string, value: any) => void;
}

export default function PointLightMenu({ selected, handleChange }: Props) {
  const colorRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<any>(null);

  const color = selected.props.color ?? "#ffffff";
  const intensity = selected.props.intensity ?? 1;
  const distance = selected.props.distance ?? 0;
  const decay = selected.props.decay ?? 1;

  function onColorChange() {
    //if same color, do nothing
    if (colorRef.current?.value === color) return;

    const debounce = setTimeout(async () => {
      const newColor = colorRef.current?.value;
      if (!newColor) return;
      handleChange("color", newColor);
    }, 5);

    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = debounce;
  }

  return (
    <>
      <MenuRow title="Color">
        <div className="h-6">
          <ColorInput
            inputRef={colorRef}
            defaultValue={color}
            onChange={onColorChange}
          />
        </div>
      </MenuRow>

      <MenuRow title="Intensity">
        <NumberInput
          updatedValue={String(round(intensity))}
          onChange={(e) =>
            handleChange("intensity", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>

      <MenuRow title="Distance">
        <NumberInput
          updatedValue={String(round(distance))}
          onChange={(e) =>
            handleChange("distance", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>

      <MenuRow title="Decay">
        <NumberInput
          updatedValue={String(round(decay))}
          onChange={(e) =>
            handleChange("decay", round(Number(e.currentTarget.value)))
          }
        />
      </MenuRow>
    </>
  );
}
