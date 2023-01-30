interface Props extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options: string[];
}

export default function SelectMenu({ options, ...rest }: Props) {
  return (
    <select
      className="w-full appearance-none rounded border border-neutral-300 bg-neutral-50 bg-arrow bg-right bg-no-repeat bg-origin-content pl-1.5 pr-1"
      {...rest}
    >
      {options.map((option) => {
        return (
          <option key={option} value={option}>
            {option}
          </option>
        );
      })}
    </select>
  );
}
