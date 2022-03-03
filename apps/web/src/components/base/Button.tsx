import { ReactChild } from "react";

interface Props {
  color?: "primary" | "gray" | "red";
  children: ReactChild;
  [ley: string]: any;
}

export default function Button({
  color = "primary",
  children,
  ...rest
}: Props) {
  const css =
    color === "primary"
      ? "bg-primary"
      : color === "gray"
      ? "bg-neutral-500"
      : "bg-red-500";

  return (
    <div
      className={`text-white py-2 px-4 hover:bg-opacity-90 shadow-md w-full ${css}
                  hover:cursor-pointer rounded-xl flex justify-center transition-all duration-150`}
      {...rest}
    >
      {children}
    </div>
  );
}
