import { useAtomValue } from "jotai";

import RoomPage from "../../../Pages/RoomPage/RoomPage";
import { roomIdAtom } from "../sidebarState";

export default function Right() {
  const roomId = useAtomValue(roomIdAtom);

  function handleJoin() {}

  return <RoomPage roomId={roomId} onClickJoin={handleJoin} />;
}
