import { useEditorStore } from "../../../../app/editor/[id]/store";

type Props = React.InputHTMLAttributes<HTMLInputElement>;

export default function NumberInput(props: Props) {
  const isPlaying = useEditorStore((state) => state.isPlaying);

  return (
    <input
      {...props}
      type="number"
      inputMode="numeric"
      disabled={isPlaying}
      className={`w-full rounded border pl-1.5 leading-snug ${
        isPlaying ? "" : "hover:bg-neutral-100 focus:bg-neutral-100"
      } `}
    />
  );
}
