interface Props {
  text?: string;
  placement?: "left" | "right" | "bottom" | "top";
  children: React.ReactChild;
}

export default function Tooltip({ text, placement = "top", children }: Props) {
  const margin =
    placement === "top"
      ? "mb-14 bottom-0 left-1/2 transform -translate-x-1/2"
      : placement === "bottom"
      ? "mt-14 top-0 left-1/2 transform -translate-x-1/2"
      : placement === "left"
      ? "mr-14 right-0 top-1/2 transform -translate-y-1/2"
      : "ml-14 left-0 top-1/2 transform -translate-y-1/2";

  return (
    <div className="group relative h-full">
      {children}

      {text && (
        <div
          className={`absolute z-20 flex flex-col items-center min-w-max ${margin}`}
        >
          <span
            className="relative px-3 py-2 text-xs leading-none rounded-lg w-full
                     text-white whitespace-no-wrap bg-black shadow-lg font-bold
                       scale-0 group-hover:scale-100 transition-all duration-300"
          >
            {text}
          </span>
        </div>
      )}
    </div>
  );
}
