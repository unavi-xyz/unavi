import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";

import DropdownMenu from "../../base/DropdownMenu";
import ObjectsMenu from "./ObjectsMenu";

export default function ObjectButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="h-full">
      <button
        onClick={(e) => {
          e.stopPropagation();
          setOpen((prev) => !prev);
        }}
        className="aspect-square h-full flex items-center justify-center
        rounded-lg hover:bg-neutral-100"
      >
        <HiOutlineCube className="text-2xl" />
      </button>

      <div className="mt-1">
        <DropdownMenu open={open} onClose={() => setOpen(false)}>
          <ObjectsMenu />
        </DropdownMenu>
      </div>
    </div>
  );
}
