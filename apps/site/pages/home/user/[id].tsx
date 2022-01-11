import { useContext } from "react";
import { Grid, Typography } from "@mui/material";
import { useRouter } from "next/router";

import { ClientContext, useProfile, useMatrixContent } from "matrix";
import { useIdenticon } from "ui";
import HomeLayout from "../../../src/layouts/HomeLayout";

export default function Id() {
  const router = useRouter();
  const id = `${router.query.id}`;

  const { client } = useContext(ClientContext);

  const identicon = useIdenticon(id);
  const profile = useProfile(client, id);
  const picture = useMatrixContent(profile?.avatar_url);

  if (!profile) {
    return (
      <Grid className="page" container direction="column" rowSpacing={4}>
        <Grid item>
          <Typography variant="h3">User not found.</Typography>
        </Grid>
      </Grid>
    );
  }

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h3">{profile.displayname ?? id}</Typography>
        <Typography variant="body1" style={{ color: "GrayText" }}>
          {id}
        </Typography>
      </Grid>

      <Grid item>
        <img
          src={picture ?? identicon}
          alt="profile picture"
          style={{
            height: "20ch",
            width: "20ch",
            border: "4px solid black",
          }}
        />
      </Grid>

      <Grid item>
        <Typography variant="h4">Worlds</Typography>
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
