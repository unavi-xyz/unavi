export enum Colors {
  sky = "group-hover:bg-sky-400",
  indigo = "group-hover:bg-indigo-400",
  purple = "group-hover:bg-purple-400",
  amber = "group-hover:bg-amber-400",
}

interface Props {
  icon: JSX.Element;
  text: string;
  color?: Colors;
  selected?: boolean;
}

export default function SidebarButton({
  icon,
  text,
  color = Colors.sky,
  selected = false,
}: Props) {
  return (
    <button
      className="group flex items-center space-x-3 p-2 rounded-lg hover:text-black
                 hover:cursor-pointer text-neutral-500 font-medium"
    >
      <div
        className={`w-7 h-7 flex items-center justify-center bg-neutral-200 rounded-md group-hover:shadow ${
          selected
            ? color === Colors.sky
              ? "bg-sky-400"
              : color === Colors.indigo
              ? "bg-indigo-400"
              : color === Colors.purple
              ? "bg-purple-400"
              : color === Colors.amber
              ? "bg-amber-400"
              : null
            : null
        } ${color}`}
        style={{ color: selected ? "black" : null }}
      >
        {icon}
      </div>
      <div style={{ color: selected ? "black" : null }}>{text}</div>
    </button>
  );
}
