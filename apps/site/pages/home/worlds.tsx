import { useContext } from "react";
import { Grid, Typography } from "@mui/material";
import { useWorlds, ClientContext } from "matrix";

import HomeLayout from "../../src/layouts/HomeLayout";
import WorldCard from "../../src/components/WorldCard";

export default function Worlds() {
  const { client } = useContext(ClientContext);

  const worlds = useWorlds(client);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Typography variant="h2">🌏 Worlds</Typography>
      </Grid>

      <Grid item container spacing={4}>
        {worlds?.map((world) => {
          return <WorldCard key={world.room_id} world={world} />;
        })}
      </Grid>
    </Grid>
  );
}

Worlds.Layout = HomeLayout;
