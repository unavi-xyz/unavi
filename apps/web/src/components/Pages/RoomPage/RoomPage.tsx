import { useAuth, useRoom } from "ceramic";

import Button from "../../Button";

interface Props {
  roomId: string;
  onClickJoin: () => void;
}

export default function RoomPage({ roomId, onClickJoin }: Props) {
  const room = useRoom(roomId);
  const { authenticated } = useAuth();

  return (
    <div className="h-full bg-neutral-100">
      <div className="w-full h-24 bg-white"></div>

      <div className="flex flex-col h-full space-y-4 p-8">
        <img
          src={room?.image}
          alt=""
          className="h-[600px] w-full object-cover rounded-md"
        />

        <div className="flex flex-col space-y-2">
          <div className="text-4xl font-medium">{room?.name}</div>
          <div className="text-xl">{room?.description}</div>

          {authenticated && (
            <div className="text-xl pt-2">
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
