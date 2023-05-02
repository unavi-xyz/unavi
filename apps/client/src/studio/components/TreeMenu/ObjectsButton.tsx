"use client";

import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";

import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import { useStudio } from "../Studio";
import ObjectsMenu from "./ObjectsMenu";

export default function ObjectsButton() {
  const [open, setOpen] = useState(false);

  const { loaded, mode } = useStudio();
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
          <HiOutlineCube className="text-2xl" />
        </IconButton>
      </DropdownTrigger>

      <DropdownContent>
        <ObjectsMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
