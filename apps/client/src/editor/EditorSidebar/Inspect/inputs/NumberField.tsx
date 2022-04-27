import { ChangeEvent } from "react";
import NumberInput from "./NumberInput";

interface Props {
  title?: string;
  step?: number;
  min?: number;
  max?: number;
  value: number;
  onChange: (value: number) => void;
}

export default function NumberField({
  title,
  step = 0.1,
  min,
  max,
  value,
  onChange,
}: Props) {
  function round(value: number) {
    if (value > max) return max;
    if (value < min) return min;
    return Math.round(value * 1000) / 1000;
  }

  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    const rounded = round(Number(e.target.value));
    onChange(rounded);
  }

  return (
    <div className="flex items-center w-full">
      <div className="w-1/4">{title}</div>
      <div className="w-1/4">
        <NumberInput step={step} value={round(value)} onChange={handleChange} />
      </div>
    </div>
  );
}
