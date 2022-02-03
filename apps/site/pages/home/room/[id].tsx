import { useEffect, useState } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { BackNavbar, getAppUrl, useIdenticon } from "ui";
import { useWorld, useRoom } from "ceramic";

import HomeLayout from "../../../src/layouts/HomeLayout";

export default function Room() {
  const router = useRouter();
  const id = router.query.id as string;

  const room = useRoom(id);
  const world = useWorld(room?.sceneStreamId);
  const identicon = useIdenticon(id);

  const [joinUrl, setJoinUrl] = useState("/");

  useEffect(() => {
    setJoinUrl(`${getAppUrl()}?room=${id}`);
  }, [id]);

  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text={room?.name} />
      </Grid>

      <Grid item>
        <img
          src={world?.image ?? identicon}
          alt="world image"
          style={{
            width: "100%",
            height: "400px",
            objectFit: "cover",
            borderBottom: "1px solid rgba(0,0,0,.1)",
          }}
        />
      </Grid>

      <Grid item sx={{ padding: 4 }}>
        <Stack spacing={2}>
          <Stack
            direction="row"
            justifyContent="space-between"
            alignItems="center"
          >
            <Link href={`/home/world/${room?.sceneStreamId}`} passHref>
              <Typography
                className="link"
                variant="h4"
                style={{ wordBreak: "break-word" }}
              >
                {room?.name}
              </Typography>
            </Link>

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

          <Typography>{world?.description}</Typography>
        </Stack>
      </Grid>
    </Grid>
  );
}

Room.Layout = HomeLayout;
