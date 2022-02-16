import { useContext, useState } from "react";
import {
  Box,
  Button,
  Divider,
  Grid,
  Stack,
  Typography,
  useMediaQuery,
} from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { HomeNavbar, theme, useIdenticon } from "ui";
import { CeramicContext, useProfile } from "ceramic";

import HomeLayout from "./HomeLayout";
import EditProfileModal from "../components/modals/EditProfileModal";

export default function UserLayout({ children }) {
  const router = useRouter();
  const id = router.query.id as string;

  const md = useMediaQuery(theme.breakpoints.up("md"));

  const identicon = useIdenticon(id);
  const { profile, imageUrl } = useProfile(id);

  const { userId } = useContext(CeramicContext);

  const [open, setOpen] = useState(false);

  return (
    <HomeLayout>
      <EditProfileModal open={open} handleClose={() => setOpen(false)} />

      <HomeNavbar text={profile?.name} back />

      <Stack spacing={2} sx={{ padding: 4 }}>
        <Stack direction="row" justifyContent="space-between" alignItems="end">
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

          {userId === id && md && (
            <Button variant="outlined" onClick={() => setOpen(true)}>
              Edit Profile
            </Button>
          )}
        </Stack>

        <Stack spacing={0}>
          <Typography variant="h6">{profile?.name}</Typography>
          <Typography
            variant="body1"
            style={{
              color: "GrayText",
              overflow: "hidden",
              textOverflow: "ellipsis",
              width: md ? null : "240px",
            }}
          >
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
