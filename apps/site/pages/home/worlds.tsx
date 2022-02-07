import { Grid } from "@mui/material";
import { BackNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Worlds() {
  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text="Worlds" />
      </Grid>
    </Grid>
  );
}

Worlds.Layout = HomeLayout;
