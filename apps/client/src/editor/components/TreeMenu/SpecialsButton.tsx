import { useState } from "react";
import { HiOutlineLightningBolt } from "react-icons/hi";

import { useEditorStore } from "@/app/editor/[id]/store";

import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import SpecialsMenu from "./SpecialsMenu";

export default function SpecialsButton() {
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);
  const isPlaying = useEditorStore((state) => state.isPlaying);

  const [open, setOpen] = useState(false);

  const disabled = !sceneLoaded || isPlaying;

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

      <DropdownContent open={open}>
        <SpecialsMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
