import { useState } from "react";
import { MdInfo } from "react-icons/md";

import NavbarButton from "../NavbarButton";
import InfoDialog from "./InfoDialog";

interface Props {
  spaceId: string;
}

export default function InfoButton({ spaceId }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <InfoDialog spaceId={spaceId} open={open} setOpen={setOpen} />

      <NavbarButton onClick={() => setOpen(true)}>
        <MdInfo className="text-[1.4rem]" />
      </NavbarButton>
    </div>
  );
}
