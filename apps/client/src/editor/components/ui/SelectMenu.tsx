import { useEditorStore } from "../../store";

interface Props extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options: string[];
}

export default function SelectMenu({ options, ...rest }: Props) {
  const isPlaying = useEditorStore((state) => state.isPlaying);

  return (
    <select
      disabled={isPlaying}
      className={`w-full appearance-none rounded border bg-arrow bg-right bg-no-repeat bg-origin-content pl-1.5 pr-1 ${
        isPlaying ? "" : "hover:bg-neutral-100 focus:bg-neutral-100"
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
