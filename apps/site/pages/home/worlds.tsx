import { Grid, Typography } from "@mui/material";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Worlds() {
  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
      <Grid item container>
        <Typography variant="h2">🌏 Worlds</Typography>
      </Grid>
    </Grid>
  );
}

Worlds.Layout = HomeLayout;
