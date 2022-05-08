import { useState } from "react";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

interface Props {
  title: string;
  children: React.ReactNode;
}

export default function CollapseMenu({ title, children }: Props) {
  const [open, setOpen] = useState(true);

  return (
    <div>
      <button
        onClick={() => setOpen((prev) => !prev)}
        className="cursor-default text-bold flex items-center space-x-2 hover:bg-neutral-100
                   rounded-md w-full px-2 py-0.5 transition-all duration-100"
      >
        {open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />}

        <div>{title}</div>
      </button>

      {open && <div className="pt-2">{children}</div>}
    </div>
  );
}
