import { useContext, useEffect, useState } from "react";
import { Button, Grid, Typography } from "@mui/material";
import Link from "next/link";
import { useRouter } from "next/router";

import { ClientContext, usePublicRoomFromId, useWorldFromRoom } from "matrix";
import { getAppUrl } from "../../../src/helpers";
import HomeLayout from "../../../src/layouts/HomeLayout";

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const { client } = useContext(ClientContext);

  const [roomURL, setRoomURL] = useState("");

  const room = usePublicRoomFromId(client, id as string);
  const world = useWorldFromRoom(client, room?.topic);

  useEffect(() => {
    const url = getAppUrl();
    setRoomURL(`${url}?room=${id}`);
  }, [id]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          🚪 {room?.name ?? id}
        </Typography>
      </Grid>

      <Grid item container columnSpacing={1}>
        <Grid item>
          <Typography variant="h6">World:</Typography>
        </Grid>
        <Grid item>
          <Link href={`/home/world/${world?.room_id}`} passHref>
            <Typography
              className="link"
              variant="h6"
              style={{ wordBreak: "break-word" }}
            >
              {world?.name}
            </Typography>
          </Link>
        </Grid>
      </Grid>

      <Grid item>
        <Button
          variant="contained"
          color="secondary"
          href={roomURL}
          target="_blank"
        >
          Join Room
        </Button>
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
