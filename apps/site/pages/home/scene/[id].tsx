import { useEffect, useState } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import Link from "next/link";
import { useRouter } from "next/router";
import { HomeNavbar, useIdenticon } from "ui";

import HomeLayout from "../../../src/layouts/HomeLayout";
import EditSceneModal from "../../../src/components/modals/EditSceneModal";

export default function Scene() {
  const router = useRouter();
  const id = router.query.id as string;

  const [preview, setPreview] = useState<string>();
  const [name, setName] = useState<string>();
  const [open, setOpen] = useState(false);

  const identicon = useIdenticon(id);

  useEffect(() => {
    setPreview(localStorage.getItem(`${id}-preview`));
    setName(localStorage.getItem(`${id}-name`));
  }, [id]);

  return (
    <Grid container direction="column">
      <EditSceneModal id={id} open={open} handleClose={() => setOpen(false)} />

      <Grid item>
        <HomeNavbar
          text={name ?? id}
          emoji="🚧"
          href="/home/scenes"
          more={() => setOpen(true)}
        />
      </Grid>

      <Grid item>
        <img
          src={preview ?? identicon}
          alt="world image"
          style={{
            width: "100%",
            height: "400px",
            objectFit: "cover",
            borderBottom: "1px solid rgba(0,0,0,.1)",
          }}
        />
      </Grid>

      <Grid item sx={{ padding: 4 }}>
        <Stack
          direction="row"
          justifyContent="space-between"
          alignItems="center"
        >
          <Typography variant="h4" style={{ wordBreak: "break-word" }}>
            {name ?? id}
          </Typography>

          <Link href={`/editor?scene=${id}`} passHref>
            <Button variant="contained" color="secondary">
              Open in Editor
            </Button>
          </Link>
        </Stack>
      </Grid>
    </Grid>
  );
}

Scene.Layout = HomeLayout;
