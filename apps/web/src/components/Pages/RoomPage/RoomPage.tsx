import { useAuth, useRoom } from "ceramic";

import Button from "../../base/Button";
import SettingsButton from "./SettingsButton/SettingsButton";

interface Props {
  spaceId: string;
  roomId: string;
  onClickJoin: () => void;
}

export default function RoomPage({ spaceId, roomId, onClickJoin }: Props) {
  const { room, controller } = useRoom(roomId);
  const { authenticated, viewerId } = useAuth();

  return (
    <div className="h-full bg-neutral-100">
      <div className="w-full h-24 bg-white"></div>

      <div className="h-full space-y-4 p-8">
        <img
          src={room?.image}
          alt=""
          className="h-[600px] w-full object-cover rounded-md"
        />

        <div className="space-y-4">
          <div className="flex justify-between">
            <div className="space-y-2">
              <div className="text-4xl font-medium">{room?.name}</div>
              <div className="text-xl">{room?.description}</div>
            </div>

            <div className="flex">
              {viewerId === controller && (
                <SettingsButton spaceId={spaceId} roomId={roomId} />
              )}
            </div>
          </div>

          {authenticated && (
            <div className="text-xl pt-2 max-w-max">
              <Button onClick={onClickJoin}>
                <div className="px-2 py-1 text-lg">Join Room</div>
              </Button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
