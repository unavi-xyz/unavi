interface Props<T extends string>
  extends React.HTMLAttributes<HTMLSelectElement> {
  label: string;
  value: T;
  options: T[];
  disabled?: boolean;
}

export function SelectInput<T extends string>({
  label,
  options,
  disabled,
  ...rest
}: Props<T>) {
  return (
    <label className="grid w-full grid-cols-4 items-center">
      <div className="block font-bold text-neutral-400">{label}</div>

      <select
        disabled={disabled}
        className="col-span-3 block rounded-md bg-neutral-800 px-2 py-1 capitalize"
        {...rest}
      >
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </label>
  );
}
