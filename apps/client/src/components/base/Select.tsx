interface Props {
  options: string[];
  inputAdornment?: string;
  title?: string;
  value?: string;
  [key: string]: any;
}

export default function Select({
  options,
  inputAdornment,
  title,
  value,
  ...rest
}: Props) {
  return (
    <div className="w-full space-y-2">
      <div className="text-lg font-bold">{title}</div>

      <div>
        <select
          value={value}
          className={`border outline-none p-2 pl-4 pr-3 rounded-lg w-full cursor-pointer
                    hover:bg-neutral-100 bg-arrow bg-no-repeat appearance-none
                      bg-right bg-origin-content`}
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
