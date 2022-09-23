interface Props extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options: string[];
}

export default function SelectMenu({ options, ...rest }: Props) {
  return (
    <select
      className="bg-arrow w-full appearance-none rounded-md bg-neutral-100 bg-right bg-no-repeat bg-origin-content pl-2.5 pr-1 shadow-inner focus:outline-none"
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
