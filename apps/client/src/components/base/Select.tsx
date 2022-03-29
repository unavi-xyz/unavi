import { ChangeEvent } from "react";

interface Props {
  options: string[];
  title?: string;
  value?: string;
  onChange?: (value: any) => void;
  [key: string]: any;
}

function capitalizeFirstLetter(string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

export function Select({ options, title, value, onChange, ...rest }: Props) {
  function handleChange(e: ChangeEvent<HTMLSelectElement>) {
    const type = e.target.value as any;
    onChange(type);
  }

  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2">{title}</div>

      <select
        {...rest}
        value={value}
        onChange={handleChange}
        className="border outline-none px-2 rounded-full w-full"
      >
        {options.map((option) => {
          return (
            <option key={option} value={option}>
              {capitalizeFirstLetter(option)}
            </option>
          );
        })}
      </select>
    </div>
  );
}
