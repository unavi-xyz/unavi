import { Triplet } from "@wired-labs/engine";
import { useEffect, useRef } from "react";

import { rgbToHex } from "../../../utils/rgb";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  rgbValue?: Triplet;
}

export default function ColorInput({ rgbValue, onChange, ...rest }: Props) {
  const displayRef = useRef<HTMLDivElement>(null);
  const colorArray = rgbValue
    ? [rgbValue[0] * 255, rgbValue[1] * 255, rgbValue[2] * 255]
    : null;
  const colorHex = colorArray ? rgbToHex(colorArray) : "#000000";

  useEffect(() => {
    if (displayRef.current && colorHex) {
      displayRef.current.style.backgroundColor = colorHex;
    }
  }, [colorHex]);

  return (
    <label className="relative">
      <div
        ref={displayRef}
        className="h-full w-full rounded-md bg-neutral-100 pl-2 shadow-inner focus:outline-none"
      />
      <input
        {...rest}
        onChange={(e) => {
          if (displayRef.current) {
            const color = e.target.value;
            displayRef.current.style.backgroundColor = color;
          }

          if (onChange) onChange(e);
        }}
        type="color"
        className="invisible absolute bottom-0 left-0"
        defaultValue={colorHex}
      />
    </label>
  );
}
