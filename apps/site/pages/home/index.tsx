import { useContext, useState } from "react";
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
import FeedItem from "../../src/components/timeline/FeedItem";

export default function Home() {
  const { authenticated } = useContext(CeramicContext);

  const [newPosts, setNewPosts] = useState<string[]>([]);

  return (
    <Grid container direction="column">
      <Grid item>
        <HomeNavbar text="Home" />
      </Grid>

      {authenticated ? (
        <div>
          <PostField setNewPosts={setNewPosts} />
          <Divider />
          {newPosts.map((streamId) => {
            return <FeedItem key={streamId} streamId={streamId} />;
          })}
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
