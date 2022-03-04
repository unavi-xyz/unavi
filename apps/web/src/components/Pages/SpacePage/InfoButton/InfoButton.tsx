import { useState } from "react";
import { MdInfo } from "react-icons/md";

import IconButton from "../../../base/IconButton";
import InfoDialog from "./InfoDialog";

interface Props {
  spaceId: string;
}

export default function InfoButton({ spaceId }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <InfoDialog spaceId={spaceId} open={open} setOpen={setOpen} />

      <IconButton onClick={() => setOpen(true)}>
        <MdInfo className="text-[1.4rem]" />
      </IconButton>
    </div>
  );
}
