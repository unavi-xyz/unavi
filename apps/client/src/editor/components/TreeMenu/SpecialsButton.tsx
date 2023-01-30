import { useState } from "react";
import { HiOutlineLightningBolt } from "react-icons/hi";

import DropdownMenu from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";

export default function SpecialsButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="aspect-square h-full">
      <IconButton onClick={() => setOpen((prev) => !prev)}>
        <HiOutlineLightningBolt className="text-2xl" />
      </IconButton>

      <div className="mt-1">
        <DropdownMenu open={open} onClose={() => setOpen(false)}>
          {/* <SpecialsMenu /> */}
        </DropdownMenu>
      </div>
    </div>
  );
}
