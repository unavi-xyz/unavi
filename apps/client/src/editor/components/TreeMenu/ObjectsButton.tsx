import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";

import DropdownMenu from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";

export default function ObjectsButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="aspect-square h-full">
      <IconButton onClick={() => setOpen((prev) => !prev)}>
        <HiOutlineCube className="text-2xl" />
      </IconButton>

      <div className="mt-1">
        <DropdownMenu open={open} onClose={() => setOpen(false)}>
          {/* <ObjectsMenu /> */}
        </DropdownMenu>
      </div>
    </div>
  );
}
