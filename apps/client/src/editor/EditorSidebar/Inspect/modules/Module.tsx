import { ReactChild, useState } from "react";
import { VscTriangleRight, VscTriangleDown } from "react-icons/vsc";

interface Props {
  title?: string;
  children: ReactChild;
}

export default function Module({ title, children }: Props) {
  const [open, setOpen] = useState(true);

  function toggleOpen() {
    setOpen((prev) => !prev);
  }

  return (
    <div className="border-b">
      <div
        onClick={toggleOpen}
        className="px-2 h-9 flex items-center hover:bg-neutral-50 cursor-default"
      >
        <div className="flex items-center space-x-2">
          <div className="text-neutral-500">
            {open ? <VscTriangleDown /> : <VscTriangleRight />}
          </div>

          <div className="text-lg">{title}</div>
        </div>
      </div>

      {open && <div className="p-3">{children}</div>}
    </div>
  );
}
