import { AiFillHome, AiFillAppstore } from "react-icons/ai";
import { useAuth, useSpaces } from "ceramic";

import SidebarButton from "./SidebarButton";
import SpaceButton from "./SpaceButton";
import CreateButton from "./CreateButton/CreateButton";
import SignInButton from "./SignInButon.tsx/SignInButton";

interface Props {
  selectedSpaceId: string;
  onSpaceClick: (streamId: string) => void;
  onHomeClick: () => void;
  onRoomsClick: () => void;
}

export default function SidebarPage({
  selectedSpaceId,
  onSpaceClick,
  onHomeClick,
  onRoomsClick,
}: Props) {
  const { authenticated, viewerId } = useAuth();
  const spaces = useSpaces(viewerId);

  return (
    <div className="h-full flex flex-col justify-between bg-primary text-white">
      <div className="flex flex-col px-3 mt-2 space-y-2">
        <div onClick={onHomeClick}>
          <SidebarButton tooltip="Home" dark icon={<AiFillHome />} />
        </div>
        <div onClick={onRoomsClick}>
          <SidebarButton tooltip="Rooms" dark icon={<AiFillAppstore />} />
        </div>

        {spaces?.map((streamId) => {
          return (
            <div
              key={streamId}
              onClick={() => {
                onSpaceClick(streamId);
              }}
            >
              <SpaceButton
                streamId={streamId}
                selected={selectedSpaceId === streamId}
              />
            </div>
          );
        })}

        {authenticated && <CreateButton />}
      </div>

      <div className="bg-black bg-opacity-10">
        <SignInButton />
      </div>
    </div>
  );
}
