import { ReactChild } from "react";
import Pressable from "./Pressable";

interface Props {
  children: ReactChild;
  [key: string]: any;
}

export default function IconButton({ children, ...rest }: Props) {
  return (
    <Pressable>
      <div
        {...rest}
        className="w-14 h-14 bg-inherit rounded-lg hover:cursor-pointer
                  hover:bg-neutral-200 flex justify-center items-center text-xl
                    transition-all duration-150 ring-neutral-200 ring-1"
      >
        {children}
      </div>
    </Pressable>
  );
}
