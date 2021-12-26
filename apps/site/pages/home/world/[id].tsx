import React, { useContext, useEffect, useState } from "react";
import { useRouter } from "next/router";
import { Grid, IconButton, Tooltip, Typography } from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";

import HomeLayout from "../../../src/layouts/HomeLayout";
import { MatrixContext } from "../../../src/matrix/MatrixProvider";
import { createRoom, getWorldInstances } from "../../../src/matrix/rooms";
import RoomCard from "../../../src/components/RoomCard";

export default function Id() {
  const router = useRouter();
  const id = `${router.query.id}`;

  const { client } = useContext(MatrixContext);

  const [rooms, setRooms] = useState([]);

  useEffect(() => {
    if (!client) return;
    getWorldInstances(client, id).then((res) => {
      setRooms(res);
    });
  }, [client, id]);

  async function handleNewRoom() {
    const { room_id } = await createRoom(client, id, "Room");
    router.push(`/home/room/${room_id}`);
  }

  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
      <Grid item container alignItems="center" columnSpacing={1}>
        <Grid item>
          <Typography variant="h4" style={{ wordBreak: "break-word" }}>
            🌍 {id}
          </Typography>
        </Grid>
        <Grid item>
          <Tooltip title="New room" placement="right">
            <IconButton onClick={handleNewRoom}>
              <AddBoxOutlinedIcon />
            </IconButton>
          </Tooltip>
        </Grid>
      </Grid>

      <Grid item>
        <Typography variant="h6">Rooms</Typography>
      </Grid>

      <Grid item container spacing={4}>
        {rooms.map((room) => {
          return <RoomCard key={room.room_id} room={room} />;
        })}
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
