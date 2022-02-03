import { Grid } from "@mui/material";
import { BackNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Rooms() {
  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text="Rooms" back={false} />
      </Grid>
    </Grid>
  );
}

Rooms.Layout = HomeLayout;
