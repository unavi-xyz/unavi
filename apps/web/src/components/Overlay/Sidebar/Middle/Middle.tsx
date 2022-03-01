import { useEffect, useState } from "react";
import { useAtom } from "jotai";

import { roomIdAtom, spaceIdAtom } from "../sidebarState";
import SpacePage from "../../../Pages/SpacePage/SpacePage";

export default function Middle() {
  const [roomId, setRoomId] = useAtom(roomIdAtom);
  const [spaceId] = useAtom(spaceIdAtom);

  const [displayedSpaceId, setDisplayedSpaceId] = useState(spaceId);

  useEffect(() => {
    if (!spaceId) return;
    setDisplayedSpaceId(spaceId);
  }, [spaceId]);

  function onRoomClick(streamId: string) {
    setRoomId(streamId);
  }

  return (
    <SpacePage
      spaceId={displayedSpaceId}
      selectedRoomId={roomId}
      onRoomClick={onRoomClick}
    />
  );
}
