import { Grid, Typography } from "@mui/material";
import { useContext, useEffect, useState } from "react";

import HomeLayout from "../../src/layouts/HomeLayout";
import { MatrixContext } from "../../src/matrix/MatrixProvider";
import { getPublicRooms } from "../../src/matrix/rooms";
import RoomCard from "../../src/components/RoomCard";

export default function Rooms() {
  const { client } = useContext(MatrixContext);

  const [rooms, setRooms] = useState([]);

  useEffect(() => {
    if (!client) return;
    getPublicRooms(client).then((res) => {
      setRooms(res);
    });
  }, [client]);

  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
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
