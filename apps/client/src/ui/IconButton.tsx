import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  selected?: boolean;
  rounded?: "full" | "small";
  cursor?: "pointer" | "default";
}

export default function IconButton({
  children,
  selected,
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

  return (
    <button
      className={`flex aspect-square h-full items-center justify-center text-2xl transition ${cursorClass} ${roundedClass} ${selectedClass}`}
      {...rest}
    >
      {children}
    </button>
  );
}
