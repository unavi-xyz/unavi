import { useContext, useState } from "react";
import { Box, Button, Divider, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { HomeNavbar, useIdenticon } from "ui";
import { CeramicContext, useProfile } from "ceramic";

import HomeLayout from "./HomeLayout";
import EditProfileModal from "../components/modals/EditProfileModal";

export default function UserLayout({ children }) {
  const router = useRouter();
  const id = router.query.id as string;

  const { id: userId } = useContext(CeramicContext);

  const [open, setOpen] = useState(false);

  const identicon = useIdenticon(id);
  const { profile, imageUrl } = useProfile(id);

  return (
    <HomeLayout>
      <span>
        <EditProfileModal open={open} handleClose={() => setOpen(false)} />

        <Grid container direction="column">
          <Grid item>
            <HomeNavbar text={profile?.name} back />
          </Grid>

          <Stack spacing={2} sx={{ padding: 4 }}>
            <Stack
              direction="row"
              justifyContent="space-between"
              alignItems="end"
            >
              <img
                src={imageUrl ?? identicon}
                alt="profile picture"
                style={{
                  height: "20ch",
                  width: "20ch",
                  border: "4px solid black",
                  objectFit: "cover",
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

          <Stack direction="row" justifyContent="space-around">
            <Tab text="Feed" href={`/home/user/${id}`} />
            <Tab text="Worlds" href={`/home/user/${id}/worlds`} />
            <Tab text="Rooms" href={`/home/user/${id}/rooms`} />
          </Stack>

          <Divider />

          <Box sx={{ padding: 4 }}>{children}</Box>
        </Grid>
      </span>
    </HomeLayout>
  );
}

function Tab({ text, href }: { text: string; href: string }) {
  const router = useRouter();
  const selected = router.asPath === href;

  return (
    <Link href={href} passHref>
      <Button
        size="large"
        fullWidth
        sx={{
          fontWeight: selected ? "bold" : undefined,
          borderRadius: 0,
          color: "black",
        }}
      >
        {text}
      </Button>
    </Link>
  );
}
