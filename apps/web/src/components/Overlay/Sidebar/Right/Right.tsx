import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";

import RoomPage from "../../../Pages/RoomPage/RoomPage";
import { roomIdAtom, spaceIdAtom } from "../sidebarState";

export default function Right() {
  const spaceId = useAtomValue(spaceIdAtom);
  const roomId = useAtomValue(roomIdAtom);

  const [displayedRoomId, setDisplayedRoomId] = useState(roomId);

  useEffect(() => {
    if (!roomId) return;
    setDisplayedRoomId(roomId);
  }, [roomId]);

  function handleJoin() {}

  return (
    <RoomPage
      spaceId={spaceId}
      roomId={displayedRoomId}
      onClickJoin={handleJoin}
    />
  );
}
