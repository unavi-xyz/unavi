interface Props {
  text: string;
  selected?: boolean;
}

export default function NavbarButton({ text, selected }: Props) {
  return <div className={`${selected && "font-bold"} text-lg`}>{text}</div>;
}
