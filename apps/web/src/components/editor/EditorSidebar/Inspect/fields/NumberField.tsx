import { ChangeEvent } from "react";
import NumberInput from "./NumberInput";

function round(value: number) {
  return Math.round(value * 1000) / 1000;
}

interface Props {
  title?: string;
  step?: number;
  value: number;
  onChange: (value: number) => void;
}

export default function NumberField({
  title,
  step = 0.1,
  value,
  onChange,
}: Props) {
  function handleChange(e: ChangeEvent<HTMLInputElement>, i: number) {
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
