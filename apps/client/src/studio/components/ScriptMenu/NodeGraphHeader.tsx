import React from "react";
import { MdClose } from "react-icons/md";

import { DropdownMenu, DropdownTrigger } from "@/src/ui/DropdownMenu";
import IconButton from "@/src/ui/IconButton";

import { useScriptExtras } from "../../hooks/useScriptExtras";
import { useStudio } from "../Studio";
import AddNodeMenu from "./AddNodeMenu";

export default function NodeGraphHeader() {
  const { scriptId, setScriptId } = useStudio();
  const extras = useScriptExtras(scriptId);

  return (
    <div className="flex w-full items-center justify-between px-4 pb-2">
      <div className="flex items-center space-x-8">
        <div className="text-lg">{extras?.name}</div>

        <DropdownMenu>
          <DropdownTrigger asChild>
            <button className="rounded-md px-2 py-0.5 text-sm text-neutral-500 transition hover:bg-neutral-200 hover:text-black active:bg-neutral-200">
              Add
            </button>
          </DropdownTrigger>

          <AddNodeMenu />
        </DropdownMenu>
      </div>

      <div className="h-8">
        <IconButton cursor="pointer" onClick={() => setScriptId(null)}>
          <MdClose />
        </IconButton>
      </div>
    </div>
  );
}
