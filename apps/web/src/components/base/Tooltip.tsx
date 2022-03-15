import { ReactChild } from "react";

interface Props {
  text?: string;
  placement?: "left" | "right";
  children: ReactChild;
}

export function Tooltip({ text, placement = "left", children }: Props) {
  const position = placement === "right" ? "left-full" : "right-full";

  return (
    <div className="group relative flex items-center justify-center">
      {children}

      {text && (
        <span
          className={`absolute p-2 mx-3 rounded-md ${position}
                   text-white bg-neutral-900 text-xs font-bold transition-all
                     duration-150 scale-0 group-hover:scale-100`}
        >
          {text}
        </span>
      )}
    </div>
  );
}
