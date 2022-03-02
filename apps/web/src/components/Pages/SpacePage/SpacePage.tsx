import { MdInfo } from "react-icons/md";
import { useAuth, useSpace } from "ceramic";

import CreateRoomButton from "./CreateRoomButton/CreateRoomButton";
import RoomList from "./RoomList/RoomList";
import SaveButton from "./SaveButton";
import NavbarButton from "./NavbarButton";

interface Props {
  spaceId: string;
  selectedRoomId: string;
  onRoomClick: (streamId: string) => void;
}

export default function SpacePage({
  spaceId,
  selectedRoomId,
  onRoomClick,
}: Props) {
  const { authenticated } = useAuth();
  const { space } = useSpace(spaceId);

  return (
    <div className="w-full h-full bg-white">
      <div className="flex items-center justify-between px-8 h-24">
        <div className="flex flex-col">
          <p className="text-sm ml-[2px]">PUBLIC SPACE</p>
          <p className="text-3xl font-medium">{space?.name}</p>
        </div>

        <div className="flex space-x-4">
          <SaveButton spaceId={spaceId} />
          <NavbarButton>
            <MdInfo className="text-[1.4rem]" />
          </NavbarButton>
        </div>
      </div>

      <div className="flex h-full w-full bg-neutral-100">
        <div className="w-full min-w-max px-8">
          <div className="flex justify-between items-center h-20">
            <p className="text-xl font-medium">Rooms</p>

            {authenticated && <CreateRoomButton spaceId={spaceId} />}
          </div>

          <RoomList
            selectedRoomId={selectedRoomId}
            roomIds={Object.values(space?.rooms ?? {})}
            onRoomClick={onRoomClick}
          />
        </div>
      </div>
    </div>
  );
}
