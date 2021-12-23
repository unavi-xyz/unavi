import { useContext } from "react";
import { useRouter } from "next/router";
import { Button, Grid, Input, TextField, Typography } from "@mui/material";

import HomeLayout from "../../../src/layouts/HomeLayout";
import { MatrixContext } from "../../../src/matrix/MatrixProvider";
import { createRoom } from "../../../src/matrix/rooms";

export default function New() {
  const router = useRouter();

  const { userId, client } = useContext(MatrixContext);

  async function handleCreateWorld() {
    const { room_id } = await createRoom(client, "Test Room");

    router.push(`/home/world/${room_id}`);
  }

  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
      <Grid item>
        <Typography variant="h4">🚧 New World</Typography>
      </Grid>

      <Grid item container>
        <Grid
          item
          xs={12}
          md={6}
          xl={4}
          container
          direction="column"
          rowSpacing={1.5}
        >
          <Grid item>
            <TextField label="World Name" style={{ width: "100%" }} />
          </Grid>

          <Grid item>
            <TextField
              label="Description"
              multiline
              style={{ width: "100%" }}
            />
          </Grid>

          <Grid item>
            <Typography variant="h6">Scene:</Typography>
            <Input type="file" style={{ width: "100%" }} />
          </Grid>
        </Grid>
      </Grid>

      <Grid item>
        <Button variant="contained" onClick={handleCreateWorld}>
          Create World
        </Button>
      </Grid>
    </Grid>
  );
}

New.Layout = HomeLayout;
