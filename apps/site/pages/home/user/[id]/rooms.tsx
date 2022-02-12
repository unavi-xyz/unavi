import { Grid, Stack } from "@mui/material";
import { useRouter } from "next/router";
import { useRooms } from "ceramic";

import UserLayout from "../../../../src/layouts/UserLayout";
import RoomCard from "../../../../src/components/cards/RoomCard";

export default function Rooms() {
  const router = useRouter();
  const id = router.query.id as string;

  const rooms = useRooms(id);

  return (
    <Stack>
      <Grid container spacing={2}>
        {rooms?.map((id) => {
          return <RoomCard key={id} id={id} />;
        })}
      </Grid>
    </Stack>
  );
}

Rooms.Layout = UserLayout;
