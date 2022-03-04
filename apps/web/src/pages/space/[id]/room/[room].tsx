import { useRouter } from "next/router";

import RoomPage from "../../../../components/Pages/RoomPage/RoomPage";
import SpaceLayout from "../../../../layouts/SpaceLayout";

export default function SpaceRoom() {
  const router = useRouter();
  const spaceId = router.query.id as string;
  const roomId = router.query.room as string;

  function handleJoin() {
    router.push(`/app?room=${roomId}`);
  }

  return (
    <RoomPage spaceId={spaceId} roomId={roomId} onClickJoin={handleJoin} />
  );
}

SpaceRoom.Layout = SpaceLayout;
