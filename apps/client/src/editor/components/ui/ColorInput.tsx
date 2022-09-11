import { useEffect, useRef } from "react";

import { rgbToHex } from "../../../utils/rgb";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  rgbValue?: [number, number, number];
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
        className="bg-neutral-100 shadow-inner rounded-md focus:outline-none pl-2 w-full h-full"
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
        className="absolute bottom-0 left-0 invisible"
        defaultValue={colorHex}
      />
    </label>
  );
}
