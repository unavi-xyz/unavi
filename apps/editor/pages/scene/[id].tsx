import { useContext, useEffect, useState } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import Link from "next/link";
import { useRouter } from "next/router";
import { ClientContext, useRoomFromId } from "matrix";
import { useIdenticon, ColorIconButton } from "ui";

import SidebarLayout from "../../src/layouts/SidebarLayout";
import SceneActions from "../../src/ui/components/SceneActions";

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const { client } = useContext(ClientContext);

  const [roomURL, setRoomURL] = useState("/");
  const [deleted, setDeleted] = useState(false);

  const room = useRoomFromId(client, id as string);
  const identicon = useIdenticon(room?.roomId);

  useEffect(() => {
    setRoomURL(`/editor?scene=${id}`);
  }, [id]);

  useEffect(() => {
    if (deleted) router.push("/");
  }, [deleted, router]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
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
              {room?.name ?? id}
            </Typography>
          </Stack>

          <Stack direction="row" alignItems="center" spacing={2}>
            <Link href={roomURL} passHref>
              <Button variant="contained" color="secondary">
                Open in Editor
              </Button>
            </Link>

            <SceneActions roomId={room?.roomId} setDeleted={setDeleted} />
          </Stack>
        </Stack>
      </Grid>

      <Grid item>
        <img
          src={identicon}
          alt="scene preview"
          style={{ width: "800px", border: "1px solid black" }}
        />
      </Grid>
    </Grid>
  );
}

Id.Layout = SidebarLayout;
