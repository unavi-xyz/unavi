import { useState } from "react";
import { BsFillGearFill } from "react-icons/bs";

import IconButton from "../../../base/IconButton";
import EditRoomDialog from "./EditRoomDialog";

interface Props {
  spaceId: string;
  roomId: string;
}

export default function SettingsButton({ spaceId, roomId }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <EditRoomDialog
        spaceId={spaceId}
        roomId={roomId}
        open={open}
        setOpen={setOpen}
      />

      <IconButton onClick={() => setOpen(true)}>
        <BsFillGearFill />
      </IconButton>
    </div>
  );
}
