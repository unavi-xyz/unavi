import { useContext, useEffect, useState } from "react";
import { ClientContext, MultiplayerContext } from "matrix";
import { RoomMember } from "matrix-js-sdk";

import OtherPlayer from "./OtherPlayer";

export function OtherPlayers() {
  const { room } = useContext(MultiplayerContext);
  const { userId } = useContext(ClientContext);

  const [roomMembers, setRoomMembers] = useState<RoomMember[]>([]);

  useEffect(() => {
    if (!room || !userId) return;

    const members = room.getJoinedMembers();
    const others = members.filter((member) => member.userId !== userId);

    setRoomMembers(others);
  }, [room, userId]);

  return (
    <group>
      {roomMembers.map((member) => {
        return <OtherPlayer key={member.userId} member={member} />;
      })}
    </group>
  );
}
