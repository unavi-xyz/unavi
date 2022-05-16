import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  selected?: boolean;
}

export default function IconButton({ children, selected, ...rest }: Props) {
  const selectedClass = selected
    ? "bg-primaryContainer text-onPrimaryContainer"
    : "hover:bg-surfaceVariant text-onSurfaceVariant";

  return (
    <button
      className={`h-full aspect-square rounded-lg flex justify-center items-center cursor-default
                  text-2xl transition ${selectedClass}`}
      {...rest}
    >
      {children}
    </button>
  );
}
