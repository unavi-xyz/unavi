import { Grid, Stack } from "@mui/material";
import { useRouter } from "next/router";
import { useWorlds } from "ceramic";

import UserLayout from "../../../../src/layouts/UserLayout";
import WorldCard from "../../../../src/components/cards/WorldCard";

export default function Worlds() {
  const router = useRouter();
  const id = router.query.id as string;

  const worlds = useWorlds(id);

  return (
    <Stack sx={{ padding: 4 }}>
      <Grid container spacing={2}>
        {worlds?.map((id) => {
          return <WorldCard key={id} id={id} />;
        })}
      </Grid>
    </Stack>
  );
}

Worlds.Layout = UserLayout;
