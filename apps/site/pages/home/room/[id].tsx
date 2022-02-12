import { useContext, useEffect, useState } from "react";
import { Button, Divider, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { HomeNavbar, useIdenticon } from "ui";
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
        <HomeNavbar
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

      <Grid item sx={{ padding: 4 }}>
        <Typography
          variant="h4"
          align="center"
          style={{ wordBreak: "break-word" }}
        >
          {name}
        </Typography>
      </Grid>

      <Grid item sx={{ padding: 4, paddingTop: 0 }}>
        <Link href={`/app?room=${id}`} passHref>
          <Button
            variant="contained"
            color="secondary"
            size="large"
            disabled={!authenticated}
            fullWidth
          >
            Join Room
          </Button>
        </Link>
      </Grid>

      <Stack direction="row" justifyContent="space-around">
        <Tab text="About" href={`/home/room/${id}`} />
      </Stack>

      <Divider />

      <Stack spacing={4} sx={{ padding: 4 }}>
        <Stack spacing={2}>
          <Stack direction="row" spacing={0.5}>
            <Typography>🌏 World:</Typography>

            <Link href={`/home/world/${room?.worldStreamId}`} passHref>
              <Typography className="link" sx={{ fontWeight: "bold" }}>
                {world?.name}
              </Typography>
            </Link>
          </Stack>

          <Stack direction="row" spacing={0.5}>
            <Typography>🔒 Room controller:</Typography>

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

function Tab({ text, href }: { text: string; href: string }) {
  const router = useRouter();
  const selected = router.asPath === href;

  return (
    <Link href={href} passHref>
      <Button
        size="large"
        fullWidth
        sx={{
          fontWeight: selected ? "bold" : undefined,
          borderRadius: 0,
          color: "black",
        }}
      >
        {text}
      </Button>
    </Link>
  );
}
