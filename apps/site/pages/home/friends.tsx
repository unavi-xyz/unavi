import { Grid } from "@mui/material";

import HomeNavbar from "../../src/components/HomeNavbar";
import HomeLayout from "../../src/layouts/HomeLayout";

export default function Friends() {
  return (
    <Grid container direction="column">
      <Grid item>
        <HomeNavbar text="Friends" />
      </Grid>
    </Grid>
  );
}

Friends.Layout = HomeLayout;
