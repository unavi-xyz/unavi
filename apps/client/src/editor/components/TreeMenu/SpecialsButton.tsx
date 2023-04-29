import { useState } from "react";
import { HiOutlineLightningBolt } from "react-icons/hi";

import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import { useEditor } from "../Editor";
import SpecialsMenu from "./SpecialsMenu";

export default function SpecialsButton() {
  const [open, setOpen] = useState(false);

  const { loaded, mode } = useEditor();
  const disabled = !loaded || mode === "play";

  return (
    <DropdownMenu
      open={open}
      onOpenChange={(value) => {
        if (value && disabled) return;
        setOpen(value);
      }}
    >
      <DropdownTrigger asChild>
        <IconButton disabled={disabled}>
          <HiOutlineLightningBolt className="text-2xl" />
        </IconButton>
      </DropdownTrigger>

      <DropdownContent>
        <SpecialsMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
