import { Grid } from "@mui/material";
import { HomeNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Worlds() {
  return (
    <Grid container direction="column">
      <Grid item>
        <HomeNavbar text="Worlds" />
      </Grid>
    </Grid>
  );
}

Worlds.Layout = HomeLayout;
