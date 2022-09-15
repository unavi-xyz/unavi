import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  selected?: boolean;
}

export default function IconButton({ children, selected, ...rest }: Props) {
  const selectedClass = selected
    ? "bg-primaryContainer text-onPrimaryContainer"
    : "hover:bg-surfaceVariant";

  return (
    <button
      className={`flex aspect-square h-full cursor-default items-center justify-center rounded-lg
                  text-2xl transition ${selectedClass}`}
      {...rest}
    >
      {children}
    </button>
  );
}
