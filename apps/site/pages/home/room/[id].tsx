import { useContext, useEffect, useState } from "react";
import {
  Box,
  Button,
  Divider,
  Grid,
  Stack,
  Typography,
  useMediaQuery,
} from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { CeramicContext, useProfile, useRoom, useWorld } from "ceramic";

import { theme } from "../../../src/theme";
import { useIdenticon } from "../../../src/hooks/useIdenticon";
import HomeLayout from "../../../src/layouts/HomeLayout";
import EditRoomModal from "../../../src/components/modals/EditRoomModal";
import HomeNavbar from "../../../src/components/HomeNavbar";

export default function Room() {
  const router = useRouter();
  const id = router.query.id as string;

  const md = useMediaQuery(theme.breakpoints.up("md"));

  const { authenticated, userId: userId } = useContext(CeramicContext);

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
    <div>
      <EditRoomModal open={open} handleClose={() => setOpen(false)} />

      <HomeNavbar
        text={name}
        emoji="🚪"
        back
        more={userId === controller ? () => setOpen(true) : undefined}
      />

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

      <Box sx={{ padding: 4 }}>
        <Typography
          variant="h4"
          align="center"
          style={{ wordBreak: "break-word" }}
        >
          {name}
        </Typography>
      </Box>

      <Divider />

      <Grid container sx={{ padding: 4 }}>
        <Grid item xs>
          <Stack spacing={4}>
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

        {md && (
          <Grid item xs>
            <Link href={`/app?room=${id}`} passHref>
              <Button
                variant="contained"
                color="secondary"
                disabled={!authenticated}
                fullWidth
              >
                Join Room
              </Button>
            </Link>
          </Grid>
        )}
      </Grid>
    </div>
  );
}

Room.Layout = HomeLayout;
