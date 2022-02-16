import { useContext, useState } from "react";
import { LoadingButton } from "@mui/lab";
import {
  Avatar,
  CircularProgress,
  Divider,
  Grid,
  InputBase,
  Link as MuiLink,
  Stack,
  Typography,
} from "@mui/material";
import Link from "next/link";
import {
  addStatus,
  ceramic,
  CeramicContext,
  loader,
  newStatus,
  Status,
  useProfile,
} from "ceramic";

import { DISCORD_URL } from "../../src/constants";
import { useIdenticon } from "../../src/hooks/useIdenticon";
import HomeNavbar from "../../src/components/HomeNavbar";
import HomeLayout from "../../src/layouts/HomeLayout";

const CHARACTER_LIMIT = 280;

export default function Home() {
  const { authenticated, userId } = useContext(CeramicContext);

  const { imageUrl } = useProfile(userId);
  const identicon = useIdenticon(userId);

  const [text, setText] = useState("");
  const [loading, setLoading] = useState(false);

  async function handlePost() {
    setLoading(true);

    const status: Status = { text };
    const streamId = await newStatus(status, loader);

    await addStatus(streamId, ceramic);

    setText("");
    setLoading(false);
  }

  return (
    <Grid container direction="column">
      <Grid item>
        <HomeNavbar text="Home" />
      </Grid>

      {authenticated ? (
        <div>
          <Stack spacing={1} sx={{ padding: 2 }}>
            <Stack direction="row" spacing={2}>
              <Link href={`/home/user/${userId}`} passHref>
                <Avatar
                  className="clickable"
                  variant="rounded"
                  src={imageUrl ?? identicon}
                  sx={{ width: "3rem", height: "3rem" }}
                />
              </Link>

              <InputBase
                fullWidth
                multiline
                placeholder="What's happening?"
                value={text}
                onChange={(e) => setText(e.target.value)}
                inputProps={{ maxLength: CHARACTER_LIMIT }}
              />
            </Stack>

            <Stack
              direction="row"
              alignItems="center"
              justifyContent="flex-end"
              spacing={3}
            >
              <CircularProgress
                variant="determinate"
                size="1.4em"
                color={
                  text.length === CHARACTER_LIMIT ? "secondary" : "primary"
                }
                value={(text.length / CHARACTER_LIMIT) * 100}
              />

              <LoadingButton
                loading={loading}
                variant="contained"
                disabled={text.length === 0}
                onClick={handlePost}
                sx={{ width: "100px" }}
              >
                Post
              </LoadingButton>
            </Stack>
          </Stack>

          <Divider />
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
