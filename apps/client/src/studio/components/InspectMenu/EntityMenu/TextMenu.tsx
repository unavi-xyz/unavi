import { useRef } from "react";

import { IEntity } from "@wired-xr/engine";

import ColorInput from "../../../../ui/base/ColorInput";
import TextArea from "../../../../ui/base/TextArea";
import { round } from "../../../../utils/round";
import MenuRow from "../MenuRow";
import NumberInput from "../NumberInput";

interface Props {
  selected: IEntity<"Text">;
  handleChange: (key: string, value: any) => void;
}

export default function TextMenu({ selected, handleChange }: Props) {
  const colorRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<any>(null);

  const text = selected.props.text ?? "";
  const fontSize = selected.props.fontSize ?? 1;
  const color = selected.props.color ?? "#000000";

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
      <MenuRow title="Text">
        <TextArea
          value={text}
          onChange={(e) => {
            handleChange("text", e.currentTarget.value);
          }}
        />
      </MenuRow>

      <MenuRow title="Font Size">
        <NumberInput
          updatedValue={String(round(fontSize))}
          onChange={(e) =>
            handleChange("fontSize", Math.max(0, round(Number(e.currentTarget.value))))
          }
        />
      </MenuRow>

      <MenuRow title="Color">
        <div className="h-6">
          <ColorInput inputRef={colorRef} defaultValue={color} onChange={onColorChange} />
        </div>
      </MenuRow>
    </>
  );
}
