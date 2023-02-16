import { useState } from "react";
import { HiOutlineLightningBolt } from "react-icons/hi";

import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import SpecialsMenu from "./SpecialsMenu";

export default function SpecialsButton() {
  const [open, setOpen] = useState(false);

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownTrigger asChild>
        <IconButton>
          <HiOutlineLightningBolt className="text-2xl" />
        </IconButton>
      </DropdownTrigger>

      <DropdownContent open={open}>
        <SpecialsMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
