import { useState } from "react";
import { MdAddBox } from "react-icons/md";

import CreateRoomDialog from "./CreateRoomDialog";

interface Props {
  spaceId: string;
}

export default function CreateRoomButton({ spaceId }) {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <CreateRoomDialog spaceId={spaceId} open={open} setOpen={setOpen} />

      <button
        onClick={() => setOpen(true)}
        className="flex items-center space-x-2 font-medium hover:bg-neutral-200
               hover:cursor-pointer px-2 py-1 rounded transition-all duration-150"
      >
        <div className="text-lg">
          <MdAddBox />
        </div>
        <p className="text-md">Create</p>
      </button>
    </div>
  );
}
