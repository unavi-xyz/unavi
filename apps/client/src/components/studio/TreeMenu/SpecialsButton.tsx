import { useState } from "react";
import { HiOutlineLightningBolt } from "react-icons/hi";

import DropdownMenu from "../../base/DropdownMenu";
import IconButton from "../../base/IconButton";
import SpecialsMenu from "./SpecialsMenu";

export default function SpecialsButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="h-full aspect-square">
      <IconButton onClick={() => setOpen((prev) => !prev)}>
        <HiOutlineLightningBolt className="text-2xl" />
      </IconButton>

      <div className="mt-1">
        <DropdownMenu open={open} onClose={() => setOpen(false)}>
          <SpecialsMenu />
        </DropdownMenu>
      </div>
    </div>
  );
}
