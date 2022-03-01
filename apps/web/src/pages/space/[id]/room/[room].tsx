import { useRouter } from "next/router";

import RoomPage from "../../../../components/Pages/RoomPage/RoomPage";
import SpaceLayout from "../../../../layouts/SpaceLayout";

export default function SpaceRoom() {
  const router = useRouter();
  const roomId = router.query.room as string;

  function handleJoin() {
    router.push(`/app?room=${roomId}`);
  }

  return <RoomPage roomId={roomId} onClickJoin={handleJoin} />;
}

SpaceRoom.Layout = SpaceLayout;
