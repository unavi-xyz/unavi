import { ReactChild } from "react";

interface Props {
  text?: string;
  children: ReactChild;
}

export default function Tooltip({ children, text }: Props) {
  return (
    <div className="group flex items-center space-x-4">
      {children}

      {text && (
        <span
          className="absolute w-auto p-2 m-2 min-w-max left-16 rounded-md shadow
                   text-white bg-neutral-900 text-xs font-bold transition-all
                     duration-150 scale-0 origin-left group-hover:scale-100"
        >
          {text}
        </span>
      )}
    </div>
  );
}
