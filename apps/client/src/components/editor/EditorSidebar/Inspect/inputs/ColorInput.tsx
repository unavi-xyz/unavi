import { ChangeEvent, useRef } from "react";

interface Props {
  id: string;
  value?: string;
  onChange?: (value: string) => void;
  [key: string]: any;
}

export default function ColorInput({ id, value, onChange }: Props) {
  const ref = useRef<HTMLInputElement>();

  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    onChange(e.target.value);
  }

  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2"></div>

      <label
        htmlFor={id}
        className="w-full h-6 rounded-full border relative hover:cursor-pointer"
        style={{ backgroundColor: value }}
      >
        <input
          ref={ref}
          id={id}
          type="color"
          className="invisible absolute"
          defaultValue={value}
          onChange={handleChange}
        />
      </label>
    </div>
  );
}
