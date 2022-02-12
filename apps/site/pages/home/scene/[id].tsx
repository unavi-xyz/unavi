import { useEffect, useState } from "react";
import { Box, Button, Stack, Typography } from "@mui/material";
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

    const newName = localStorage.getItem(`${id}-name`);
    setName(newName?.length > 0 ? newName : id);
  }, [id]);

  return (
    <div>
      <EditSceneModal id={id} open={open} handleClose={() => setOpen(false)} />

      <HomeNavbar
        text={name}
        emoji="🚧"
        href="/home/scenes"
        more={() => setOpen(true)}
      />

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

      <Box sx={{ padding: 4 }}>
        <Stack spacing={4}>
          <Typography
            variant="h4"
            align="center"
            style={{ wordBreak: "break-word" }}
          >
            {name}
          </Typography>

          <Link href={`/editor?scene=${id}`} passHref>
            <Button variant="contained" color="secondary" fullWidth>
              Open in Editor
            </Button>
          </Link>
        </Stack>
      </Box>
    </div>
  );
}

Scene.Layout = HomeLayout;
