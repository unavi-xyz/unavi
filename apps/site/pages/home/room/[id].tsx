import { Button, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { BackNavbar, useIdenticon } from "ui";
import { useRoom, useScene } from "ceramic";

import HomeLayout from "../../../src/layouts/HomeLayout";

export default function Room() {
  const router = useRouter();
  const id = router.query.id as string;

  const identicon = useIdenticon(id);
  const room = useRoom(id);
  const { scene } = useScene(room?.sceneStreamId);

  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text={room?.name} back />
      </Grid>

      <Grid item>
        <img
          src={scene?.image ?? identicon}
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

            <Link href={`/app?room=${id}`} passHref>
              <Button variant="contained" color="secondary" size="large">
                Join Room
              </Button>
            </Link>
          </Stack>

          <Typography>{scene?.description}</Typography>
        </Stack>
      </Grid>
    </Grid>
  );
}

Room.Layout = HomeLayout;
