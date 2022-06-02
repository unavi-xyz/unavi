import { useState } from "react";
import { MdOutlineLightMode } from "react-icons/md";

import DropdownMenu from "../../base/DropdownMenu";
import IconButton from "../../base/IconButton";
import LightsMenu from "./LightsMenu";

export default function LightsButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="h-full aspect-square">
      <IconButton
        onClick={(e) => {
          e.stopPropagation();
          setOpen((prev) => !prev);
        }}
      >
        <MdOutlineLightMode className="text-2xl" />
      </IconButton>

      <div className="mt-1">
        <DropdownMenu open={open} onClose={() => setOpen(false)}>
          <LightsMenu />
        </DropdownMenu>
      </div>
    </div>
  );
}
