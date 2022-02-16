import { useContext, useEffect, useState } from "react";
import { Box, Button, Divider, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { CeramicContext, useWorld, useProfile } from "ceramic";

import HomeLayout from "./HomeLayout";
import EditWorldModal from "../components/modals/EditWorldModal";
import HomeNavbar from "../components/HomeNavbar";
import { useIdenticon } from "../hooks/useIdenticon";

export default function WorldLayout({ children }) {
  const router = useRouter();
  const id = router.query.id as string;

  const { userId } = useContext(CeramicContext);

  const identicon = useIdenticon(id);
  const { world, controller } = useWorld(id);
  const { profile } = useProfile(controller);

  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");

  useEffect(() => {
    if (!world) return;
    if (world?.name?.length > 0) setName(world.name);
    else setName(id);
  }, [id, world]);

  return (
    <HomeLayout>
      <EditWorldModal open={open} handleClose={() => setOpen(false)} />

      <HomeNavbar
        text={world?.name}
        emoji="🌏"
        back
        more={userId === controller ? () => setOpen(true) : undefined}
      />

      <img
        src={world?.image ?? identicon}
        alt="picture of world"
        style={{
          width: "100%",
          height: "400px",
          objectFit: "cover",
          borderBottom: "1px solid rgba(0,0,0,.1)",
        }}
      />

      <Stack alignItems="center" sx={{ padding: 4 }}>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          {name}
        </Typography>

        {controller && (
          <Stack direction="row" alignItems="center" spacing={1}>
            <Typography variant="h6">By</Typography>
            <Link href={`/home/user/${controller}/worlds`} passHref>
              <Typography className="link" variant="h6">
                {profile ? profile?.name ?? controller : ""}
              </Typography>
            </Link>
          </Stack>
        )}
      </Stack>

      <Stack direction="row" justifyContent="space-around">
        <Tab text="About" href={`/home/world/${id}`} />
        <Tab text="Rooms" href={`/home/world/${id}/rooms`} />
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
