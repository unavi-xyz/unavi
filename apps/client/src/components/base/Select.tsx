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

      <div>
        <select
          value={value}
          className={`outline-none ${thinCss} pl-4 pr-3 rounded-lg w-full transition
                      hover:bg-opacity-70 bg-arrow bg-no-repeat appearance-none
                      bg-right bg-origin-content bg-surface text-onSurface`}
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
    </div>
  );
}
