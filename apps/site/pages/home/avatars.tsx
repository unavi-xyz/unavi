import { Grid } from "@mui/material";
import { HomeNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Avatars() {
  return (
    <Grid container direction="column">
      <Grid item>
        <HomeNavbar text="Avatars" />
      </Grid>
    </Grid>
  );
}

Avatars.Layout = HomeLayout;
