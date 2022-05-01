interface Props {
  icon?: React.ReactNode;
  text?: string;
  hoverable?: boolean;
}

export default function Chip({ icon, text, hoverable }: Props) {
  const hoverableClass = hoverable ? "hover:bg-neutral-100" : "";

  return (
    <div className={`px-3 py-0.5 rounded-full border ${hoverableClass}`}>
      <div className="flex items-center space-x-2">
        {icon && <div className="text-neutral-800">{icon}</div>}
        {text && <div>{text}</div>}
      </div>
    </div>
  );
}
