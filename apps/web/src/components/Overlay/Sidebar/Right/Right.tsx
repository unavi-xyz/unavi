import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";

import RoomPage from "../../../Pages/RoomPage/RoomPage";
import { roomIdAtom } from "../sidebarState";

export default function Right() {
  const roomId = useAtomValue(roomIdAtom);

  const [displayedRoomId, setDisplayedRoomId] = useState(roomId);

  useEffect(() => {
    if (!roomId) return;
    setDisplayedRoomId(roomId);
  }, [roomId]);

  function handleJoin() {}

  return <RoomPage roomId={displayedRoomId} onClickJoin={handleJoin} />;
}
