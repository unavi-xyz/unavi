import { ReactChild } from "react";
import { ImSpinner2 } from "react-icons/im";

interface Props {
  color?: "black" | "gray" | "red";
  loading?: boolean;
  disabled?: boolean;
  children: ReactChild;
  [key: string]: any;
}

export function Button({
  color = "black",
  loading = false,
  disabled = false,
  children,
  ...rest
}: Props) {
  const colorCss =
    disabled || loading
      ? "bg-neutral-400"
      : color === "black"
      ? "bg-black"
      : color === "gray"
      ? "bg-neutral-500"
      : "bg-red-500";

  const cursorCss = disabled ? "cursor-not-allowed" : "cursor-pointer";

  return (
    <div
      className={`text-white h-10 px-4 w-full rounded-xl ${colorCss} ${cursorCss}
                  flex items-center justify-center transition-all duration-300`}
      {...rest}
    >
      {loading ? <ImSpinner2 className="animate-spin" /> : children}
    </div>
  );
}
