import { useContext, useEffect, useState } from "react";
import { Box, Button, Divider, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { BackNavbar, useIdenticon } from "ui";
import { CeramicContext, useWorld, useProfile } from "ceramic";

import HomeLayout from "./HomeLayout";
import EditWorldModal from "../components/modals/EditWorldModal";

export default function WorldLayout({ children }) {
  const router = useRouter();
  const id = router.query.id as string;

  const { id: userId } = useContext(CeramicContext);

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
      <Grid container direction="column">
        <EditWorldModal open={open} handleClose={() => setOpen(false)} />

        <Grid item>
          <BackNavbar
            text={world?.name}
            emoji="🌏"
            back
            more={userId === controller ? () => setOpen(true) : undefined}
          />
        </Grid>

        <Grid item>
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
        </Grid>

        <Grid spacing={2} item sx={{ padding: 4 }}>
          <Stack alignItems="center">
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
        </Grid>

        <Stack direction="row" justifyContent="space-around">
          <Tab text="About" href={`/home/world/${id}`} />
          <Tab text="Rooms" href={`/home/world/${id}/rooms`} />
        </Stack>

        <Divider />

        <Box sx={{ padding: 4 }}>{children}</Box>
      </Grid>
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
