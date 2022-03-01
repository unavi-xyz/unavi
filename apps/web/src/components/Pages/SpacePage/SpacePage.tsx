import { MdInfo } from "react-icons/md";
import { BsFillGearFill, BsFillPeopleFill } from "react-icons/bs";
import { useAuth, useSpace } from "ceramic";

import CreateRoomButton from "./CreateRoomButton/CreateRoomButton";
import RoomList from "../../RoomList/RoomList";

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
  const space = useSpace(spaceId);

  return (
    <div className="w-full h-full bg-white">
      <div className="flex items-center justify-between px-8 h-24">
        <div className="flex flex-col">
          <p className="text-sm ml-[2px]">PUBLIC SPACE</p>
          <p className="text-3xl font-medium">{space?.name}</p>
        </div>

        <div className="flex space-x-4">
          <NavbarButton icon={<MdInfo className="text-[1.4rem]" />} />
          <NavbarButton icon={<BsFillPeopleFill />} />
          <NavbarButton icon={<BsFillGearFill />} />
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

function NavbarButton({ icon = <></> }) {
  return (
    <div
      className="w-14 h-14 bg-neutral-100 rounded-lg hover:cursor-pointer
               hover:bg-neutral-200 flex justify-center items-center text-xl
                 transition-all duration-150 shadow"
    >
      {icon}
    </div>
  );
}
