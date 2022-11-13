import { ButtonHTMLAttributes } from "react";

import Spinner from "./Spinner";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  selected?: boolean;
  loading?: boolean;
  rounded?: "full" | "small";
  cursor?: "pointer" | "default";
}

export default function IconButton({
  children,
  selected,
  loading,
  rounded = "small",
  cursor = "default",
  ...rest
}: Props) {
  const selectedClass = selected
    ? "bg-primaryContainer text-onPrimaryContainer"
    : "hover:bg-surfaceVariant";

  const roundedClass = rounded === "full" ? "rounded-full" : "rounded-lg";

  const cursorClass =
    cursor === "pointer" ? "cursor-pointer" : "cursor-default";

  const loadingClass = loading ? "opacity-0" : null;

  return (
    <button
      className={`flex aspect-square h-full items-center justify-center text-2xl transition ${cursorClass} ${roundedClass} ${selectedClass}`}
      {...rest}
    >
      {loading && (
        <div className="absolute">
          <Spinner />
        </div>
      )}

      <div className={`${loadingClass}`}>{children}</div>
    </button>
  );
}
