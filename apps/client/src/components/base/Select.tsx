interface Props {
  options: string[];
  title?: string;
  value?: string;
  [key: string]: any;
}

function capitalizeFirstLetter(string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

export function Select({ options, title, value, ...rest }: Props) {
  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2">{title}</div>

      <select
        {...rest}
        value={value}
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
