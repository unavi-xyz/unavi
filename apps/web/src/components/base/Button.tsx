import { ReactChild } from "react";

interface Props {
  color?: "primary" | "gray";
  children: ReactChild;
  [ley: string]: any;
}

export default function Button({
  color = "primary",
  children,
  ...rest
}: Props) {
  const css = color === "primary" ? "bg-primary" : "bg-neutral-500";

  return (
    <button
      className={`text-white py-2 px-4 hover:bg-opacity-90 shadow-md w-full ${css}
                  hover:cursor-pointer rounded-xl flex justify-center transition-all duration-150`}
      {...rest}
    >
      {children}
    </button>
  );
}
