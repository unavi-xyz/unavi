import { useContext, useState } from "react";
import { Drawer, IconButton, Stack, useMediaQuery } from "@mui/material";
import MenuIcon from "@mui/icons-material/Menu";
import Link from "next/link";
import { CeramicContext } from "ceramic";

import { SidebarButton, theme } from "..";
import UserProfileButton from "./UserProfileButton";

export function Sidebar() {
  const [openDrawer, setOpenDrawer] = useState(false);

  const md = useMediaQuery(theme.breakpoints.up("md"));

  if (!md) {
    return (
      <Stack alignItems="center" style={{ height: "100%" }}>
        <IconButton onClick={() => setOpenDrawer(true)}>
          <MenuIcon />
        </IconButton>

        <Drawer
          anchor="left"
          open={openDrawer}
          onClose={() => setOpenDrawer(false)}
        >
          <Stack justifyContent="space-between" style={{ height: "100%" }}>
            <Stack spacing={1.5} sx={{ width: "240px", padding: 1 }}>
              <Buttons />
            </Stack>

            <UserProfileButton />
          </Stack>
        </Drawer>
      </Stack>
    );
  }

  return (
    <Stack justifyContent="space-between" style={{ height: "100%" }}>
      <Stack spacing={1.5}>
        <Link href="/home" passHref>
          <h1>The Wired</h1>
        </Link>

        <Buttons />
      </Stack>

      <UserProfileButton />
    </Stack>
  );
}

function Buttons() {
  const { authenticated, userId } = useContext(CeramicContext);

  return (
    <>
      <SidebarButton emoji="🏠" text="Home" href="/home" />
      <SidebarButton emoji="🌏" text="Worlds" href="/home/worlds" />
      <SidebarButton emoji="🚪" text="Rooms" href="/home/rooms" />
      <SidebarButton emoji="🤝" text="Friends" href="/home/friends" />
      <SidebarButton emoji="💃" text="Avatars" href="/home/avatars" />
      <SidebarButton
        emoji="💎"
        text="Profile"
        href={`/home/user/${userId}`}
        disabled={!authenticated}
      />
      <SidebarButton emoji="🚧" text="Editor" href="/home/scenes" />
    </>
  );
}
