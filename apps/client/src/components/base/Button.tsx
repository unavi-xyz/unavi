import { ReactChild } from "react";
import { ImSpinner2 } from "react-icons/im";

interface Props {
  color?: "black" | "gray" | "red";
  loading?: boolean;
  children: ReactChild;
  [key: string]: any;
}

export function Button({
  color = "black",
  loading = false,
  children,
  ...rest
}: Props) {
  const colorCss =
    color === "black"
      ? "bg-black"
      : color === "gray"
      ? "bg-neutral-500"
      : "bg-red-500";

  const loadingCss = loading ? "bg-opacity-40" : "hover:cursor-pointer";

  return (
    <div
      className={`text-white h-10 px-4 w-full  rounded-xl ${colorCss} ${loadingCss}
                  flex items-center justify-center transition-all duration-150`}
      {...rest}
    >
      {loading ? <ImSpinner2 className="animate-spin" /> : children}
    </div>
  );
}
