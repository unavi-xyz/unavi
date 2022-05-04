import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";

import DropdownMenu from "../../base/DropdownMenu";
import IconButton from "../../base/IconButton";
import ObjectsMenu from "./ObjectsMenu";

export default function ObjectButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="h-full">
      <IconButton
        onClick={(e) => {
          e.stopPropagation();
          setOpen((prev) => !prev);
        }}
      >
        <HiOutlineCube className="text-2xl" />
      </IconButton>

      <div className="mt-1">
        <DropdownMenu open={open} onClose={() => setOpen(false)}>
          <ObjectsMenu />
        </DropdownMenu>
      </div>
    </div>
  );
}
