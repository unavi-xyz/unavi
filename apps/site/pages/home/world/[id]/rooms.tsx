import { useContext } from "react";
import { Grid, Stack } from "@mui/material";
import { useRouter } from "next/router";
import { CeramicContext, useWorld, useRooms } from "ceramic";

import WorldLayout from "../../../../src/layouts/WorldLayout";
import RoomCard from "../../../../src/components/cards/RoomCard";
import NewRoomCard from "../../../../src/components/cards/NewRoomCard";

export default function Rooms() {
  const router = useRouter();
  const id = router.query.id as string;

  const { authenticated } = useContext(CeramicContext);

  const { controller } = useWorld(id);
  const rooms = useRooms(controller);

  return (
    <Stack spacing={1}>
      <Grid container spacing={2}>
        {authenticated && <NewRoomCard worldId={id} />}

        {rooms?.map((roomId) => {
          return <RoomCard key={roomId} id={roomId} worldFilter={id} />;
        })}
      </Grid>
    </Stack>
  );
}

Rooms.Layout = WorldLayout;
