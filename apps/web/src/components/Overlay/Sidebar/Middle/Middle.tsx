import { useEffect } from "react";
import { useSpace } from "ceramic";
import { useAtom } from "jotai";

import { roomIdAtom, showRightAtom, spaceIdAtom } from "../sidebarState";

import SpacePage from "../../../Pages/SpacePage/SpacePage";

export default function Middle() {
  const [, setShowRight] = useAtom(showRightAtom);
  const [roomId, setRoomId] = useAtom(roomIdAtom);
  const [spaceId] = useAtom(spaceIdAtom);

  const space = useSpace(spaceId);

  useEffect(() => {
    if (Object.values(space?.rooms ?? {}).includes(roomId)) {
      setShowRight(true);
    } else {
      setShowRight(false);
    }
  }, [roomId, setShowRight, space]);

  function onRoomClick(streamId: string) {
    setRoomId(streamId);
    setShowRight(true);
  }

  return (
    <SpacePage
      spaceId={spaceId}
      selectedRoomId={roomId}
      onRoomClick={onRoomClick}
    />
  );
}
