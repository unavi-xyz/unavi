import { useContext } from "react";
import { Grid, IconButton, Stack, Tooltip, Typography } from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";
import { useRouter } from "next/router";
import useSWR from "swr";
import { ClientContext, createRoom, getWorldInstances, useWorld } from "matrix";

import HomeLayout from "../../../src/layouts/HomeLayout";
import RoomCard from "../../../src/components/RoomCard";
import Link from "next/link";

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const { client, loggedIn } = useContext(ClientContext);

  async function fetcher() {
    const rooms = await getWorldInstances(client, id as string);
    return rooms;
  }

  const world = useWorld(client, id as string);

  const { data } = useSWR(`instances-${id}`, fetcher, {
    refreshInterval: 30000,
  });

  async function handleNewRoom() {
    if (!client || !id || !world) return;
    const { room_id } = await createRoom(client, `${id}`, world.name);
    router.push(`/home/room/${room_id}`);
  }

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          🌍 {world?.name}
        </Typography>
      </Grid>

      <Grid item container>
        <Grid item xs>
          <Stack spacing={1}>
            <Stack direction="row" alignItems="center" spacing={1}>
              <Typography variant="h6">Author:</Typography>
              <Link href={`/home/user/${world?.author?.userId}`} passHref>
                <Typography className="link" variant="h6">
                  {world?.author?.displayName}
                </Typography>
              </Link>
            </Stack>

            <Stack direction="row" alignItems="center" spacing={1}>
              <Typography variant="h6">Description:</Typography>
              <Typography variant="h6">{world?.description}</Typography>
            </Stack>
          </Stack>
        </Grid>
        <Grid item xs></Grid>
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
        {data?.map((room) => {
          return <RoomCard key={room.room_id} room={room} />;
        })}
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
