import React, { useContext } from "react";
import { useRouter } from "next/router";
import { Button, Grid, Typography } from "@mui/material";

import HomeLayout from "../../../../src/layouts/HomeLayout";
import { createRoom } from "../../../../src/matrix/rooms";
import { MatrixContext } from "../../../../src/matrix/MatrixProvider";

export default function NewRoom() {
  const router = useRouter();
  const { id } = router.query;

  const { client } = useContext(MatrixContext);

  async function handleCreateRoom() {
    const { room_id } = await createRoom(client, "Test Room 44");

    router.push(`/home/room/${room_id}`);
  }

  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
      <Grid item>
        <Typography variant="h4">🚪 New Room</Typography>
      </Grid>

      <Grid item>
        <Typography variant="h6">
          World: <span style={{ color: "GrayText" }}>{id}</span>
        </Typography>
        <Typography variant="h6">
          Privacy: <span style={{ color: "GrayText" }}>Public</span>
        </Typography>
      </Grid>

      <Grid item>
        <Button variant="contained" onClick={handleCreateRoom}>
          Create Room
        </Button>
      </Grid>
    </Grid>
  );
}

NewRoom.Layout = HomeLayout;
