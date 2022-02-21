import { useContext } from "react";
import {
  Divider,
  Grid,
  Link as MuiLink,
  Stack,
  Typography,
} from "@mui/material";
import { CeramicContext } from "ceramic";

import { DISCORD_URL } from "../../src/constants";
import HomeNavbar from "../../src/components/HomeNavbar";
import HomeLayout from "../../src/layouts/HomeLayout";
import Timeline from "../../src/components/timeline/Timeline";
import PostField from "../../src/components/timeline/PostField";

export default function Home() {
  const { authenticated } = useContext(CeramicContext);

  return (
    <Grid container direction="column">
      <Grid item>
        <HomeNavbar text="Home" />
      </Grid>

      {authenticated ? (
        <div>
          <PostField />
          <Divider />
          <Timeline />
        </div>
      ) : (
        <Grid item sx={{ padding: 4 }}>
          <Stack spacing={1}>
            <Typography variant="h4">Welcome!</Typography>
            <Typography>
              The platform is currently in heavy development. Please reach out
              to us on{" "}
              <MuiLink href={DISCORD_URL} target="_blank" color="secondary">
                Discord
              </MuiLink>{" "}
              if you have any questions!
            </Typography>
          </Stack>
        </Grid>
      )}
    </Grid>
  );
}

Home.Layout = HomeLayout;
