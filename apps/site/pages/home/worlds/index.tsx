import { useContext, useEffect, useState } from "react";
import { Grid, IconButton, Tooltip, Typography } from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";
import Link from "next/link";

import { getWorlds, ClientContext } from "matrix";
import HomeLayout from "../../../src/layouts/HomeLayout";
import WorldCard from "../../../src/components/WorldCard";

export default function Worlds() {
  const { client, loggedIn } = useContext(ClientContext);

  const [worlds, setWorlds] = useState([]);

  useEffect(() => {
    if (!client) return;
    getWorlds(client).then((res) => {
      setWorlds(res);
    });
  }, [client]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item container alignItems="center" columnSpacing={1}>
        <Grid item>
          <Typography variant="h2">🌏 Worlds</Typography>
        </Grid>
        <Grid item>
          {loggedIn && (
            <Link href="/home/worlds/new" passHref>
              <Tooltip title="New world" placement="right">
                <IconButton>
                  <AddBoxOutlinedIcon />
                </IconButton>
              </Tooltip>
            </Link>
          )}
        </Grid>
      </Grid>

      <Grid item container spacing={4}>
        {worlds.map((world) => {
          return <WorldCard key={world.room_id} world={world} />;
        })}
      </Grid>
    </Grid>
  );
}

Worlds.Layout = HomeLayout;
