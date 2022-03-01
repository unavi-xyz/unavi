import { useState } from "react";
import { useAtom } from "jotai";

import { roomIdAtom, spaceIdAtom } from "../sidebarState";
import SidebarPage from "../../../Pages/SidebarPage/SidebarPage";

export default function Left() {
  const [, setSpaceId] = useAtom(spaceIdAtom);
  const [, setRoomId] = useAtom(roomIdAtom);

  const [selectedSpaceId, setSelectedSpaceId] = useState<string>();

  function handleSpaceClick(streamId: string) {
    setSpaceId(streamId);
    setSelectedSpaceId(streamId);
    setRoomId(undefined);
  }

  function handleHomeClick() {
    setSpaceId(undefined);
    setSelectedSpaceId(undefined);
  }

  function handleRoomsClick() {
    setSpaceId(undefined);
    setSelectedSpaceId(undefined);
  }

  return (
    <SidebarPage
      selectedSpaceId={selectedSpaceId}
      onSpaceClick={handleSpaceClick}
      onHomeClick={handleHomeClick}
      onRoomsClick={handleRoomsClick}
    />
  );
}
