interface Props {
  text?: string;
  placement?: "left" | "right" | "bottom" | "top";
  children: JSX.Element;
}

export default function Tooltip({ text, placement = "bottom", children }: Props) {
  const margin =
    placement === "top"
      ? "mb-14 bottom-0 left-1/2 transform -translate-x-1/2"
      : placement === "bottom"
      ? "mt-14 top-0 left-1/2 transform -translate-x-1/2"
      : placement === "left"
      ? "mr-8 right-0 top-1/2 transform -translate-y-1/2"
      : "ml-8 left-0 top-1/2 transform -translate-y-1/2";

  return (
    <div className="group relative inline">
      {children}

      {text && (
        <div
          className={`absolute z-20 flex min-w-max scale-0 flex-col items-center transition duration-300 group-hover:scale-100 ${margin}`}
        >
          <span className="relative w-full rounded-lg bg-neutral-800 px-3 py-2 text-xs font-bold leading-none text-white shadow-lg">
            {text}
          </span>
        </div>
      )}
    </div>
  );
}
