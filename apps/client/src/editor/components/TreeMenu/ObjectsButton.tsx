import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";

import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import { useEditorStore } from "../../store";
import ObjectsMenu from "./ObjectsMenu";

export default function ObjectsButton() {
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  const [open, setOpen] = useState(false);

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownTrigger asChild>
        <IconButton disabled={!sceneLoaded}>
          <HiOutlineCube className="text-2xl" />
        </IconButton>
      </DropdownTrigger>

      <DropdownContent open={open}>
        <ObjectsMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
