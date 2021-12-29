import { useContext, useState } from "react";
import { Button, Grid, Input, TextField, Typography } from "@mui/material";
import { useRouter } from "next/router";

import { MatrixContext, createWorld } from "matrix";
import HomeLayout from "../../../src/layouts/HomeLayout";

export default function New() {
  const router = useRouter();

  const { client } = useContext(MatrixContext);

  const [name, setName] = useState("");

  async function handleCreateWorld() {
    const { room_id } = await createWorld(
      client,
      name.length === 0 ? "World" : name
    );
    router.push(`/home/world/${room_id}`);
  }

  function handleNameChange(e) {
    setName(e.target.value);
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
            <TextField
              label="World Name"
              value={name}
              onChange={handleNameChange}
              style={{ width: "100%" }}
            />
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
