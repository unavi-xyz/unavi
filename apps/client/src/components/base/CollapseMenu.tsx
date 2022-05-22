import { useState } from "react";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

interface Props {
  title: string;
  open: boolean;
  toggle: () => void;
  children: React.ReactNode;
}

export default function CollapseMenu({ title, open, toggle, children }: Props) {
  return (
    <div>
      <button
        onClick={toggle}
        className="cursor-default text-bold flex items-center space-x-2
                   rounded-md w-full px-2 py-0.5 transition
                   hover:bg-secondaryContainer"
      >
        {open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />}

        <div>{title}</div>
      </button>

      {open && <div className="pt-2">{children}</div>}
    </div>
  );
}
