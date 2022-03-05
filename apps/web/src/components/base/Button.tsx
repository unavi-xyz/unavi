import { ReactChild } from "react";

interface Props {
  color?: "primary" | "gray" | "red";
  children: ReactChild;
  [ley: string]: any;
}

export function Button({ color = "primary", children, ...rest }: Props) {
  return (
    <div
      className={`text-white py-2 px-4 w-full hover:cursor-pointer
                  rounded-xl flex justify-center transition-all duration-150
                  ${
                    color === "primary"
                      ? "bg-primary"
                      : color === "gray"
                      ? "bg-neutral-500"
                      : "bg-red-500"
                  }
                 `}
      {...rest}
    >
      {children}
    </div>
  );
}
