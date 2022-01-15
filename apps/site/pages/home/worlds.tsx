import { Grid, Typography } from "@mui/material";
import { IPublicRoomsChunkRoom } from "matrix-js-sdk";
import { getWorlds, getGuestClient } from "matrix";

import HomeLayout from "../../src/layouts/HomeLayout";
import WorldCard from "../../src/components/WorldCard";

interface Props {
  worlds: IPublicRoomsChunkRoom[];
}
export default function Worlds({ worlds }: Props) {
  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item container alignItems="center" columnSpacing={1}>
        <Grid item>
          <Typography variant="h2">🌏 Worlds</Typography>
        </Grid>
        <Grid item></Grid>
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

export async function getStaticProps() {
  const client = await getGuestClient();
  const worlds = await getWorlds(client);

  return { props: { worlds } };
}
