import { useContext } from "react";
import { Grid, IconButton, Stack, Tooltip, Typography } from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import Link from "next/link";
import { useRouter } from "next/router";
import useSWR from "swr";
import { customAlphabet } from "nanoid";
import { ColorIconButton } from "ui";
import {
  ClientContext,
  createRoom,
  getPublicRoom,
  getWorldInstances,
} from "matrix";

import HomeLayout from "../../../src/layouts/HomeLayout";
import RoomCard from "../../../src/components/RoomCard";

const nanoid = customAlphabet("1234567890", 8);

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const { client, loggedIn } = useContext(ClientContext);

  async function fetcher(worldId) {
    const world = await getPublicRoom(client, worldId as string, true);
    const rooms = await getWorldInstances(client, worldId as string);

    return { world, rooms };
  }

  const { data } = useSWR(id, fetcher);

  async function handleNewRoom() {
    if (!client || !id || !data?.world) return;
    const name = `${data?.world.name}#${nanoid()}`;
    const { room_id } = await createRoom(client, `${id}`, name);
    router.push(`/home/room/${room_id}`);
  }

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Stack direction="row" alignItems="center">
          <Link href="/home/worlds" passHref>
            <span>
              <ColorIconButton>
                <ArrowBackIosNewIcon />
              </ColorIconButton>
            </span>
          </Link>
          <Typography variant="h4" style={{ wordBreak: "break-word" }}>
            {data?.world.name}
          </Typography>
        </Stack>
      </Grid>

      <Grid item container alignItems="center" columnSpacing={1}>
        <Grid item>
          <Typography variant="h5">Rooms</Typography>
        </Grid>
        <Grid item>
          {loggedIn && (
            <Tooltip title="New room" placement="right">
              <IconButton onClick={handleNewRoom}>
                <AddBoxOutlinedIcon />
              </IconButton>
            </Tooltip>
          )}
        </Grid>
      </Grid>

      <Grid item container spacing={4}>
        {data?.rooms.map((room) => {
          return <RoomCard key={room.room_id} room={room} />;
        })}
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
