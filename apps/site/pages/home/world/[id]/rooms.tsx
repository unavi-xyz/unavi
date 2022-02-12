import { useContext } from "react";
import { Button } from "@mui/material";
import { useRouter } from "next/router";
import { CeramicContext, Room, loader } from "ceramic";

import WorldLayout from "../../../../src/layouts/WorldLayout";

const roomModel = require("ceramic/models/Room/model.json");
const roomSchemaId = roomModel.schemas.Scene;

export default function Rooms() {
  const router = useRouter();
  const id = router.query.id as string;

  const { authenticated } = useContext(CeramicContext);

  async function handleNewRoom() {
    const room: Room = { worldStreamId: id };

    //create tile
    const stream = await loader.create(room, { schema: roomSchemaId });
    const streamId = stream.id.toString();

    const url = `/home/room/${streamId}`;
    router.push(url);
  }

  return (
    <Button
      variant="contained"
      color="secondary"
      onClick={handleNewRoom}
      disabled={!authenticated}
      fullWidth
    >
      Create Room
    </Button>
  );
}

Rooms.Layout = WorldLayout;
