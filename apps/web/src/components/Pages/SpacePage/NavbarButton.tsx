import { ReactChild } from "react";
import Pressable from "../../base/Pressable";

interface Props {
  children: ReactChild;
  [key: string]: any;
}

export default function NavbarButton({ children, ...rest }) {
  return (
    <Pressable>
      <div
        {...rest}
        className="w-14 h-14 bg-neutral-100 rounded-lg hover:cursor-pointer
        hover:bg-neutral-200 flex justify-center items-center text-xl
        transition-all duration-150 shadow"
      >
        {children}
      </div>
    </Pressable>
  );
}
