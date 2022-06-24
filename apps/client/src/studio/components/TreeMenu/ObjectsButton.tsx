import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";

import DropdownMenu from "../../../ui/base/DropdownMenu";
import IconButton from "../../../ui/base/IconButton";
import ObjectsMenu from "./ObjectsMenu";

export default function ObjectsButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="h-full aspect-square">
      <IconButton onClick={() => setOpen((prev) => !prev)}>
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
