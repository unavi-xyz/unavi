import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  selected?: boolean;
}

export default function IconButton({ children, selected, ...rest }: Props) {
  const selectedClass = selected ? "bg-neutral-100" : "";

  return (
    <button
      className={`h-full aspect-square rounded-lg flex justify-center items-center cursor-default
                hover:bg-neutral-100 text-2xl transition-all duration-100 ${selectedClass}`}
      {...rest}
    >
      {children}
    </button>
  );
}
