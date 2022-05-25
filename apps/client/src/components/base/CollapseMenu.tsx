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
        className="group cursor-default text-bold flex items-center
                   rounded-md w-full px-2 transition space-x-2
                   hover:bg-secondaryContainer"
      >
        <div className="text-outline group-hover:text-inherit transition">
          {open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />}
        </div>

        <div>{title}</div>
      </button>

      {open && <div className="pt-1">{children}</div>}
    </div>
  );
}
