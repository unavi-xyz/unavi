interface Props {
  options: string[];
  inputAdornment?: string;
  title?: string;
  value?: string;
  thin?: boolean;
  [key: string]: any;
}

export default function Select({
  options,
  inputAdornment,
  title,
  value,
  thin,
  ...rest
}: Props) {
  const thinCss = thin ? undefined : "py-2";

  return (
    <div className="w-full space-y-2">
      <div className="text-lg font-bold">{title}</div>

      <select
        value={value}
        className={`w-full appearance-none rounded-lg bg-surface bg-arrow bg-right bg-no-repeat
                    bg-origin-content pl-4 pr-3
                    text-onSurface outline-none transition hover:shadow ${thinCss}`}
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
