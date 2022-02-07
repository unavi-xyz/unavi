import { Grid } from "@mui/material";
import { BackNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Friends() {
  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text="Friends" />
      </Grid>
    </Grid>
  );
}

Friends.Layout = HomeLayout;
