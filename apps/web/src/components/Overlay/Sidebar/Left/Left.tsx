import { useState } from "react";
import { useAtom } from "jotai";

import { showMiddleAtom, spaceIdAtom } from "../sidebarState";

import SidebarPage from "../../../Pages/SidebarPage/SidebarPage";

export default function Left() {
  const [, setShowMiddle] = useAtom(showMiddleAtom);
  const [, setSpaceId] = useAtom(spaceIdAtom);

  const [selectedSpaceId, setSelectedSpaceId] = useState<string>();

  function handleSpaceClick(streamId: string) {
    setSpaceId(streamId);
    setSelectedSpaceId(streamId);
    setShowMiddle(true);
  }

  function handleHomeClick() {
    setShowMiddle(false);
    setSelectedSpaceId(undefined);
  }

  function handleRoomsClick() {
    setShowMiddle(false);
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
