import { Grid, Typography } from "@mui/material";

import { getGuestClient, getPublicRooms } from "matrix";
import HomeLayout from "../../src/layouts/HomeLayout";
import RoomCard from "../../src/components/RoomCard";
import { IPublicRoomsChunkRoom } from "matrix-js-sdk";

interface Props {
  rooms: IPublicRoomsChunkRoom[];
}
export default function Rooms({ rooms }: Props) {
  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h2">🚪 Rooms</Typography>
      </Grid>

      <Grid item container spacing={4}>
        {rooms.map((room) => {
          return <RoomCard key={room.room_id} room={room} />;
        })}
      </Grid>
    </Grid>
  );
}

Rooms.Layout = HomeLayout;

export async function getStaticProps() {
  const client = await getGuestClient();
  const rooms = await getPublicRooms(client);

  return { props: { rooms } };
}
