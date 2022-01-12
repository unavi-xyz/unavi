import { useContext, useEffect, useState } from "react";
import { Button, Grid, IconButton, Stack, Typography } from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";
import { useRouter } from "next/router";

import { ClientContext, useRoomFromId } from "matrix";
import { useIdenticon } from "ui";
import SidebarLayout from "../../src/layouts/SidebarLayout";

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const { client } = useContext(ClientContext);

  const [roomURL, setRoomURL] = useState("/");

  const room = useRoomFromId(client, id as string);
  const identicon = useIdenticon(room?.roomId);

  useEffect(() => {
    setRoomURL(`/editor?scene=${id}`);
  }, [id]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item style={{ maxWidth: "800px" }}>
        <Stack
          direction="row"
          alignItems="center"
          justifyContent="space-between"
        >
          <Typography variant="h4" style={{ wordBreak: "break-word" }}>
            {room?.name ?? id}
          </Typography>

          <Stack direction="row" alignItems="center" spacing={2}>
            <Link href={roomURL} passHref>
              <Button variant="contained" color="secondary">
                Open in Editor
              </Button>
            </Link>

            <IconButton>
              <MoreHorizIcon />
            </IconButton>
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
