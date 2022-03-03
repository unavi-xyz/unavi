import { useState } from "react";
import { BsFillGearFill } from "react-icons/bs";

import NavbarButton from "../NavbarButton";
import EditSpaceDialog from "./EditSpaceDialog";

interface Props {
  spaceId: string;
}

export default function SettingsButton({ spaceId }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <EditSpaceDialog spaceId={spaceId} open={open} setOpen={setOpen} />

      <NavbarButton onClick={() => setOpen(true)}>
        <BsFillGearFill />
      </NavbarButton>
    </div>
  );
}
