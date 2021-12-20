import { Grid, Typography } from "@mui/material";

import HomeLayout from "../../src/layouts/HomeLayout";

export default function Home() {
  return (
    <Grid className="container" container justifyContent="center">
      <Grid
        item
        sm={8}
        md={6}
        container
        direction="column"
        rowSpacing={4}
        justifyContent="center"
        style={{ height: "100vh" }}
      >
        <Grid item>
          <Typography variant="h2">Home</Typography>
        </Grid>
      </Grid>
    </Grid>
  );
}

Home.Layout = HomeLayout;
