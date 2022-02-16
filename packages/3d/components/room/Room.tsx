import { ReactChild } from "react";
import { Physics } from "@react-three/cannon";
import { useRoom } from "ceramic";

import { Multiplayer, MultiplayerProvider } from "../..";
import { World } from "../world/World";

interface Props {
  roomId: string;
  userId: string;
  children: ReactChild | ReactChild[];
}

export function Room({ roomId, userId, children }: Props) {
  const { room } = useRoom(roomId);

  if (!room) return null;

  return (
    <MultiplayerProvider roomId={roomId} userId={userId}>
      <Physics>
        <World worldId={room.worldStreamId} />
        <Multiplayer userId={userId} />
        <group>{children}</group>
      </Physics>
    </MultiplayerProvider>
  );
}
