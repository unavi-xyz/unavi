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
        className="text-bold hover:bg-surfaceVariant group flex w-full
                   cursor-default items-center space-x-2 rounded-md
                   px-2"
      >
        <div className="text-outline group-hover:text-inherit">
          {open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />}
        </div>

        <div>{title}</div>
      </button>

      {open && <div className="pt-2">{children}</div>}
    </div>
  );
}
