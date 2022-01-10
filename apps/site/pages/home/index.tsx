import { Grid, Typography } from "@mui/material";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Home() {
  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item container>
        <Typography variant="h2">🏠 Home</Typography>
      </Grid>
    </Grid>
  );
}

Home.Layout = HomeLayout;
