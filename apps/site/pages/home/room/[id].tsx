import { useEffect, useState } from "react";
import { Button, Grid, Typography } from "@mui/material";
import Link from "next/link";
import { IPublicRoomsChunkRoom } from "matrix-js-sdk";
import { getGuestClient, getPublicRoom, parseRoomTopic } from "matrix";

import { getAppUrl } from "../../../src/helpers";
import HomeLayout from "../../../src/layouts/HomeLayout";

interface Props {
  room: IPublicRoomsChunkRoom;
  world: IPublicRoomsChunkRoom;
}
export default function Id({ room, world }: Props) {
  const [joinUrl, setJoinUrl] = useState("");

  useEffect(() => {
    const url = getAppUrl();
    setJoinUrl(`${url}?room=${room.room_id}`);
  }, [room.room_id]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          🚪 {room.name}
        </Typography>
      </Grid>

      <Grid item container columnSpacing={1}>
        <Grid item>
          <Typography variant="h6">World:</Typography>
        </Grid>
        <Grid item>
          <Link href={`/home/world/${world.room_id}`} passHref>
            <Typography
              className="link"
              variant="h6"
              style={{ wordBreak: "break-word" }}
            >
              {world.name}
            </Typography>
          </Link>
        </Grid>
      </Grid>

      <Grid item>
        <Button
          variant="contained"
          color="secondary"
          href={joinUrl}
          target="_blank"
        >
          Join Room
        </Button>
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;

export async function getServerSideProps(context) {
  const id = context.params.id;

  const client = await getGuestClient();
  const room = await getPublicRoom(client, id);

  const worldId = parseRoomTopic(room.topic);
  const world = await getPublicRoom(client, worldId, true);

  return { props: { room, world } };
}
