import { useState } from "react";
import { AiOutlinePlus } from "react-icons/ai";

import CreateDialog from "./CreateSpaceDialog";
import SidebarButton from "../SidebarButton";

export default function CreateButton() {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <CreateDialog open={open} setOpen={setOpen} />

      <div onClick={() => setOpen(true)}>
        <SidebarButton
          tooltip="Create"
          selected={open}
          icon={<AiOutlinePlus />}
        />
      </div>
    </div>
  );
}
