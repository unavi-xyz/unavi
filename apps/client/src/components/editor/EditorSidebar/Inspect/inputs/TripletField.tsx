import { ChangeEvent } from "react";
import { Triplet } from "@react-three/cannon";
import { degToRad, radToDeg } from "three/src/math/MathUtils";

import NumberInput from "./NumberInput";

function round(value: number) {
  return Math.round(value * 1000) / 1000;
}

interface Props {
  title?: string;
  radians?: boolean;
  step?: number;
  value: Triplet;
  onChange: (value: Triplet) => void;
}

export default function TripletField({
  title,
  radians,
  step = 0.1,
  value,
  onChange,
}: Props) {
  function handleChange(e: ChangeEvent<HTMLInputElement>, i: number) {
    const rounded = round(Number(e.target.value));

    if (radians) value[i] = degToRad(rounded);
    else value[i] = rounded;

    onChange(value);
  }

  function getHandleChange(i: number) {
    return (e: ChangeEvent<HTMLInputElement>) => handleChange(e, i);
  }

  function getValue(i: number) {
    if (radians) return round(radToDeg(value[i]));
    else return round(value[i]);
  }

  return (
    <div className="flex items-center w-full">
      <div className="w-1/4">{title}</div>

      <div className="w-1/4">
        <NumberInput
          title="x"
          step={step}
          value={getValue(0)}
          onChange={getHandleChange(0)}
        />
      </div>

      <div className="w-1/4">
        <NumberInput
          title="y"
          step={step}
          value={getValue(1)}
          onChange={getHandleChange(1)}
        />
      </div>

      <div className="w-1/4">
        <NumberInput
          title="z"
          step={step}
          value={getValue(2)}
          onChange={getHandleChange(2)}
        />
      </div>
    </div>
  );
}
