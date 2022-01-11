import { useContext, useEffect, useState } from "react";
import { Button, Grid, Typography } from "@mui/material";
import Link from "next/link";
import { useRouter } from "next/router";

import { ClientContext, useRoomFromId } from "matrix";
import SidebarLayout from "../../src/layouts/SidebarLayout";

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const { client } = useContext(ClientContext);

  const [roomURL, setRoomURL] = useState("/");

  const room = useRoomFromId(client, id as string);

  useEffect(() => {
    setRoomURL(`/editor?scene=${id}`);
  }, [id]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          {room?.name ?? id}
        </Typography>
      </Grid>

      <Grid item>
        <Link href={roomURL} passHref>
          <Button variant="contained" color="secondary">
            Open in Editor
          </Button>
        </Link>
      </Grid>
    </Grid>
  );
}

Id.Layout = SidebarLayout;
