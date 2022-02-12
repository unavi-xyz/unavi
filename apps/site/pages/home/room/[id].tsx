import { useContext, useEffect, useState } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { BackNavbar, useIdenticon } from "ui";
import { CeramicContext, useProfile, useRoom, useWorld } from "ceramic";

import HomeLayout from "../../../src/layouts/HomeLayout";
import EditRoomModal from "../../../src/components/modals/EditRoomModal";

export default function Room() {
  const router = useRouter();
  const id = router.query.id as string;

  const { authenticated, id: userId } = useContext(CeramicContext);

  const identicon = useIdenticon(id);
  const { room, controller } = useRoom(id);
  const { world } = useWorld(room?.worldStreamId);
  const { profile } = useProfile(controller);

  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");

  useEffect(() => {
    if (!room || !world) return;

    if (room?.name?.length > 0) setName(room.name);
    else if (world?.name?.length > 0) setName(world.name);
    else setName(id);
  }, [id, room, world]);

  return (
    <Grid container direction="column">
      <EditRoomModal open={open} handleClose={() => setOpen(false)} />

      <Grid item>
        <BackNavbar
          text={name}
          emoji="🚪"
          back
          more={userId === controller ? () => setOpen(true) : undefined}
        />
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

      <Stack spacing={4} sx={{ padding: 4 }}>
        <Link href={`/app?room=${id}`} passHref>
          <Button
            variant="contained"
            color="secondary"
            size="large"
            fullWidth
            disabled={!authenticated}
          >
            Join Room
          </Button>
        </Link>

        <Stack spacing={2}>
          <Typography variant="h4" style={{ wordBreak: "break-word" }}>
            {name}
          </Typography>

          <Stack direction="row" spacing={0.5}>
            <Typography>World:</Typography>

            <Link href={`/home/world/${room?.worldStreamId}`} passHref>
              <Typography className="link" sx={{ fontWeight: "bold" }}>
                {world?.name}
              </Typography>
            </Link>
          </Stack>

          <Stack direction="row" spacing={0.5}>
            <Typography>Owner:</Typography>

            <Link href={`/home/user/${controller}`} passHref>
              <Typography className="link" sx={{ fontWeight: "bold" }}>
                {profile?.name ?? controller}
              </Typography>
            </Link>
          </Stack>
        </Stack>
      </Stack>
    </Grid>
  );
}

Room.Layout = HomeLayout;
