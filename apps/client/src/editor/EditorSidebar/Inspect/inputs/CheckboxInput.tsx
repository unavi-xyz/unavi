import { ChangeEvent } from "react";

interface Props {
  checked: boolean;
  onChange?: (value: boolean) => void;
  [key: string]: any;
}

export default function CheckboxInput({ checked, onChange }: Props) {
  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    const checked = e.target.checked;
    onChange(checked);
  }

  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2"></div>

      <input
        type="checkbox"
        checked={checked}
        className="border outline-none pl-2 rounded-full"
        onChange={handleChange}
      />
    </div>
  );
}
