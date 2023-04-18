import { useEditor } from "../Editor";

interface Props extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options: string[];
}

export default function SelectMenu({ options, disabled, ...rest }: Props) {
  const { mode } = useEditor();

  const isDisabled = disabled || mode === "play";

  return (
    <select
      disabled={isDisabled}
      className={`w-full appearance-none rounded-md border border-neutral-200 bg-arrow bg-right bg-no-repeat bg-origin-content pl-1.5 pr-1 capitalize ${
        isDisabled ? "" : "hover:bg-neutral-100 focus:bg-neutral-100"
      }`}
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
