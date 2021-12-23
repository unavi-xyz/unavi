import React, { useContext, useEffect } from "react";
import Link from "next/link";
import { useRouter } from "next/router";
import { Grid, IconButton, Tooltip, Typography } from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";

import HomeLayout from "../../../../src/layouts/HomeLayout";
import { MatrixContext } from "../../../../src/matrix/MatrixProvider";
import { getRoom } from "../../../../src/matrix/rooms";

export default function Id() {
  const router = useRouter();
  const id = `${router.query.id}`;

  const { client } = useContext(MatrixContext);

  useEffect(() => {
    if (!client) return;
    getRoom(client, id).then((res) => {
      console.log(res);
    });
  }, [client, id]);

  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
      <Grid item container alignItems="center" columnSpacing={1}>
        <Grid item>
          <Typography variant="h4" style={{ wordBreak: "break-word" }}>
            🌍 {id}
          </Typography>
        </Grid>
        <Grid item>
          <Link href={`/home/world/${id}/new-room`} passHref>
            <Tooltip title="New room" placement="right">
              <IconButton>
                <AddBoxOutlinedIcon />
              </IconButton>
            </Tooltip>
          </Link>
        </Grid>
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
