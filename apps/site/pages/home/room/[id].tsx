import { useContext, useEffect, useState } from "react";
import { Button, Grid, Typography, Stack } from "@mui/material";
import Link from "next/link";
import { useRouter } from "next/router";
import { ClientContext, useRoom, useWorld, useRoomAvatar } from "matrix";
import { useIdenticon } from "ui";

import { getAppUrl } from "../../../src/helpers";
import HomeLayout from "../../../src/layouts/HomeLayout";

export default function Id() {
  const router = useRouter();
  const id = router.query.id;

  const { client } = useContext(ClientContext);

  const [joinUrl, setJoinUrl] = useState("");

  const room = useRoom(client, id as string);
  const world = useWorld(client, room?.world);

  const avatar = useRoomAvatar(client, room?.room.chunk);
  const identicon = useIdenticon(id as string);

  useEffect(() => {
    const url = getAppUrl();
    setJoinUrl(`${url}?room=${id}`);
  }, [id]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Stack
          direction="row"
          alignItems="center"
          justifyContent="space-between"
        >
          <Typography variant="h4" style={{ wordBreak: "break-word" }}>
            🚪 {room?.name}
          </Typography>

          <Button
            variant="contained"
            color="secondary"
            size="large"
            href={joinUrl}
            target="_blank"
          >
            Join Room
          </Button>
        </Stack>
      </Grid>

      <Grid item container>
        <Grid item xs>
          <Stack direction="row" spacing={1}>
            <Typography variant="h6">World:</Typography>
            <Link href={`/home/world/${world?.room?.roomId}`} passHref>
              <Typography
                className="link"
                variant="h6"
                style={{ wordBreak: "break-word" }}
              >
                {world?.name}
              </Typography>
            </Link>
          </Stack>
        </Grid>

        <Grid item xs>
          <img
            src={avatar ?? identicon}
            alt="world image"
            style={{
              border: "2px solid black",
              width: "100%",
              height: "400px",
              objectFit: "cover",
            }}
          />
        </Grid>
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
