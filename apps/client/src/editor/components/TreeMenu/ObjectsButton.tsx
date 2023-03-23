import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";

import { useEditorStore } from "../../../../app/editor/[id]/store";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import ObjectsMenu from "./ObjectsMenu";

export default function ObjectsButton() {
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
          <HiOutlineCube className="text-2xl" />
        </IconButton>
      </DropdownTrigger>

      <DropdownContent open={open}>
        <ObjectsMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
