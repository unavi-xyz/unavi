import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  selected?: boolean;
}

export default function IconButton({ children, selected, ...rest }: Props) {
  const selectedClass = selected ? "bg-neutral-100" : "";

  return (
    <button
      className={`h-full aspect-square rounded-lg flex justify-center items-center
                hover:bg-neutral-100 cursor-pointer text-2xl ${selectedClass}
                  transition-all duration-100`}
      {...rest}
    >
      {children}
    </button>
  );
}
