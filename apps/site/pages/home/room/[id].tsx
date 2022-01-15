import { useContext, useEffect, useState } from "react";
import { Button, Grid, Typography } from "@mui/material";
import Link from "next/link";
import { ClientContext, getPublicRoom, parseRoomTopic } from "matrix";

import { getAppUrl } from "../../../src/helpers";
import HomeLayout from "../../../src/layouts/HomeLayout";
import useSWR from "swr";
import { useRouter } from "next/router";

export default function Id() {
  const router = useRouter();
  const id = router.query.id;

  const { client } = useContext(ClientContext);

  const [joinUrl, setJoinUrl] = useState("");

  async function fetcher(id) {
    const room = await getPublicRoom(client, id);
    const worldId = parseRoomTopic(room.topic);
    const world = await getPublicRoom(client, worldId, true);
    return { room, world };
  }

  const { data } = useSWR(id, fetcher);

  useEffect(() => {
    const url = getAppUrl();
    setJoinUrl(`${url}?room=${data?.room.room_id}`);
  }, [data?.room.room_id]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          🚪 {data?.room.name}
        </Typography>
      </Grid>

      <Grid item container columnSpacing={1}>
        <Grid item>
          <Typography variant="h6">World:</Typography>
        </Grid>
        <Grid item>
          <Link href={`/home/world/${data?.world.room_id}`} passHref>
            <Typography
              className="link"
              variant="h6"
              style={{ wordBreak: "break-word" }}
            >
              {data?.world.name}
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
