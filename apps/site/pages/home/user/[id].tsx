import { useContext, useState } from "react";
import { Button, Divider, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import { BackNavbar, useIdenticon } from "ui";
import { CeramicContext, useProfile, useWorlds } from "ceramic";

import HomeLayout from "../../../src/layouts/HomeLayout";
import WorldCard from "../../../src/components/WorldCard";
import EditProfileModal from "../../../src/components/EditProfileModal";

export default function User() {
  const router = useRouter();
  const id = router.query.id as string;

  const { id: userId } = useContext(CeramicContext);

  const [open, setOpen] = useState(false);

  const identicon = useIdenticon(id);
  const worlds = useWorlds(id);
  const { profile } = useProfile(id);

  return (
    <span>
      <EditProfileModal open={open} handleClose={() => setOpen(false)} />

      <Grid container direction="column">
        <Grid item>
          <BackNavbar text={profile?.name} back />
        </Grid>

        <Stack spacing={2} sx={{ padding: 4 }}>
          <Stack
            direction="row"
            justifyContent="space-between"
            alignItems="end"
          >
            <img
              src={identicon}
              alt="profile picture"
              style={{
                height: "20ch",
                width: "20ch",
                border: "4px solid black",
              }}
            />

            {userId === id && (
              <Button variant="outlined" onClick={() => setOpen(true)}>
                Edit Profile
              </Button>
            )}
          </Stack>

          <Stack spacing={0}>
            <Typography variant="h6">{profile?.name}</Typography>
            <Typography variant="body1" style={{ color: "GrayText" }}>
              {id}
            </Typography>
          </Stack>

          <Typography variant="body1">{profile?.description}</Typography>
        </Stack>

        <Divider />

        <Stack sx={{ padding: 4 }}>
          {worlds && (
            <Typography variant="h4" sx={{ marginBottom: 4 }}>
              Worlds
            </Typography>
          )}
          <Grid container spacing={2}>
            {worlds?.map((id) => {
              return <WorldCard key={id} id={id} />;
            })}
          </Grid>
        </Stack>
      </Grid>
    </span>
  );
}

User.Layout = HomeLayout;
