import { ReactChild } from "react";

interface Props {
  color?: "black" | "gray" | "red";
  children: ReactChild;
  [ley: string]: any;
}

export function Button({ color = "black", children, ...rest }: Props) {
  return (
    <div
      className={`text-white py-2 px-4 w-full hover:cursor-pointer
                  rounded-xl flex justify-center transition-all duration-150
                  ${
                    color === "black"
                      ? "bg-black"
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
