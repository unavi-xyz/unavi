import { useContext, useState } from "react";
import { LoadingButton } from "@mui/lab";
import { useRouter } from "next/router";
import { CeramicContext, Room, loader, newRoom } from "ceramic";

import WorldLayout from "../../../../src/layouts/WorldLayout";

export default function Rooms() {
  const router = useRouter();
  const id = router.query.id as string;

  const { authenticated } = useContext(CeramicContext);

  const [loading, setLoading] = useState(false);

  async function handleNewRoom() {
    setLoading(true);

    const room: Room = { worldStreamId: id };
    const streamId = await newRoom(room, loader);

    const url = `/home/room/${streamId}`;
    router.push(url);
  }

  return (
    <LoadingButton
      loading={loading}
      variant="contained"
      color="secondary"
      onClick={handleNewRoom}
      disabled={!authenticated}
      fullWidth
    >
      Create Room
    </LoadingButton>
  );
}

Rooms.Layout = WorldLayout;
