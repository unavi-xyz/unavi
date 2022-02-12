import { Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import { useWorld } from "ceramic";

import WorldLayout from "../../../../src/layouts/WorldLayout";

export default function World() {
  const router = useRouter();
  const id = router.query.id as string;

  const { world } = useWorld(id);

  return (
    <Stack spacing={1}>
      {world?.description && <Typography variant="h6">Description</Typography>}
      <Typography>{world?.description}</Typography>
    </Stack>
  );
}

World.Layout = WorldLayout;
