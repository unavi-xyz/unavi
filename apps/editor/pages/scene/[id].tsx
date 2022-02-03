import { useEffect, useState } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import Link from "next/link";
import { useRouter } from "next/router";
import { useIdenticon, ColorIconButton } from "ui";

import SidebarLayout from "../../src/layouts/SidebarLayout";
import SceneActions from "../../src/ui/components/SceneActions";

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const [preview, setPreview] = useState<null | string>();
  const [name, setName] = useState<null | string>();

  const identicon = useIdenticon(id as string);

  useEffect(() => {
    setPreview(localStorage.getItem(`${id}-preview`));
    setName(localStorage.getItem(`${id}-name`));
  }, [id]);

  return (
    <Grid container direction="column" rowSpacing={4} sx={{ padding: 4 }}>
      <Grid item style={{ maxWidth: "800px" }}>
        <Stack
          direction="row"
          alignItems="center"
          justifyContent="space-between"
        >
          <Stack direction="row" alignItems="center" spacing={1}>
            <Link href="/" passHref>
              <span>
                <ColorIconButton>
                  <ArrowBackIosNewIcon />
                </ColorIconButton>
              </span>
            </Link>

            <Typography variant="h4" style={{ wordBreak: "break-word" }}>
              {name ?? id}
            </Typography>
          </Stack>

          <Stack direction="row" alignItems="center" spacing={2}>
            <Link href={`/editor?scene=${id}`} passHref>
              <Button variant="contained" color="secondary">
                Open in Editor
              </Button>
            </Link>

            <SceneActions id={id as string} />
          </Stack>
        </Stack>
      </Grid>

      <Grid item>
        <img
          src={preview ?? identicon}
          alt="scene preview"
          style={{ width: "800px", border: "1px solid black" }}
        />
      </Grid>
    </Grid>
  );
}

Id.Layout = SidebarLayout;
