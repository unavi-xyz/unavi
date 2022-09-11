interface Props {
  title: string;
  selected: string;
  setSelected: (tab: string) => void;
}

export default function PanelTab({ title, selected, setSelected }: Props) {
  const selectedCss = selected === title ? "bg-neutral-200" : null;

  return (
    <button
      onClick={() => setSelected(title)}
      className={`${selectedCss} px-3 py-1 rounded hover:bg-neutral-200 transition w-full`}
    >
      {title}
    </button>
  );
}
