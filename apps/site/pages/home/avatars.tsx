import { Grid } from "@mui/material";
import { BackNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Avatars() {
  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text="Avatars" />
      </Grid>
    </Grid>
  );
}

Avatars.Layout = HomeLayout;
