import { Grid, Link, Stack, Typography } from "@mui/material";
import { BackNavbar } from "ui";

import { DISCORD_URL } from "../../src/constants";
import HomeLayout from "../../src/layouts/HomeLayout";

export default function Home() {
  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text="Home" back={false} />
      </Grid>

      <Grid item sx={{ padding: 4 }}>
        <Stack spacing={1}>
          <Typography variant="h4">Welcome!</Typography>
          <Typography>
            The platform is currently in heavy development. Please reach out to
            us on{" "}
            <Link href={DISCORD_URL} target="_blank" color="secondary">
              Discord
            </Link>{" "}
            if you have any issues!
          </Typography>
        </Stack>
      </Grid>
    </Grid>
  );
}

Home.Layout = HomeLayout;
