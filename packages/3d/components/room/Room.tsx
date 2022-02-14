import { useRoom } from "ceramic";

import { Multiplayer, Player } from "../..";
import { World } from "../world/World";

interface Props {
  roomId: string;
}

export function Room({ roomId }: Props) {
  const { room } = useRoom(roomId);

  if (!room) return null;

  return (
    <group>
      <Multiplayer roomId={roomId} />
      <World worldId={room.worldStreamId} />
    </group>
  );
}
