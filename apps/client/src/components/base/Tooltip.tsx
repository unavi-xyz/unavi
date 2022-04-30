interface Props {
  text?: string;
  placement?: "left" | "right" | "bottom" | "top";
  children: React.ReactChild;
}

export default function Tooltip({ text, placement = "left", children }: Props) {
  const margin =
    placement === "top"
      ? "mb-14 bottom-0"
      : placement === "bottom"
      ? "mt-14 top-0"
      : placement === "left"
      ? "mr-14 right-0 translate-y-1/3"
      : "ml-14 left-0 translate-y-1/3";

  return (
    <div className="group relative flex flex-col items-center">
      {children}

      {text && (
        <div
          className={`absolute flex flex-col items-center min-w-max ${margin}`}
        >
          <span
            className="relative z-10 px-3 py-2 text-xs leading-none rounded-lg w-full
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
