interface Props extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options: string[];
  inputAdornment?: string;
  title?: string;
  outline?: boolean;
}

export default function Select({
  options,
  inputAdornment,
  title,
  outline = false,
  ...rest
}: Props) {
  const outlineClass = outline ? "border" : "";

  return (
    <div className="w-full space-y-2">
      <div className="text-lg font-bold">{title}</div>
      <select
        className={`w-full appearance-none rounded-lg bg-surface bg-arrow bg-right bg-no-repeat bg-origin-content py-2 pl-4 pr-3 text-onSurface outline-none transition hover:shadow ${outlineClass}`}
        {...rest}
      >
        {options.map((option) => {
          return (
            <option key={option} value={option}>
              {inputAdornment}
              {option}
            </option>
          );
        })}
      </select>
    </div>
  );
}
