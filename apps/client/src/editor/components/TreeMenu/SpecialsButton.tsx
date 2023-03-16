import { useState } from "react";
import { HiOutlineLightningBolt } from "react-icons/hi";

import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import { useEditorStore } from "../../store";
import SpecialsMenu from "./SpecialsMenu";

export default function SpecialsButton() {
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  const [open, setOpen] = useState(false);

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownTrigger asChild>
        <IconButton disabled={!sceneLoaded}>
          <HiOutlineLightningBolt className="text-2xl" />
        </IconButton>
      </DropdownTrigger>

      <DropdownContent open={open}>
        <SpecialsMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
