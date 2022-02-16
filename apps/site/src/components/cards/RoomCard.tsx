import { useEffect, useState } from "react";
import { Skeleton, Typography } from "@mui/material";
import { useRoom, useWorld } from "ceramic";

import { useIdenticon } from "../../hooks/useIdenticon";
import BasicCard from "./BasicCard";

interface Props {
  id: string;
  worldFilter?: string;
}

export default function RoomCard({ id, worldFilter }: Props) {
  const { room } = useRoom(id);

  const { world } = useWorld(room?.worldStreamId);
  const identicon = useIdenticon(id);

  const [name, setName] = useState("");

  useEffect(() => {
    if (!room || !world) return;

    if (room?.name?.length > 0) setName(room.name);
    else if (world?.name?.length > 0) setName(world.name);
    else setName(id);
  }, [id, room, world]);

  if (worldFilter && room?.worldStreamId !== worldFilter) return null;

  return (
    <BasicCard href={`/home/room/${id}`} image={world?.image ?? identicon}>
      {name ? (
        <Typography style={{ wordBreak: "break-word" }}>🚪 {name}</Typography>
      ) : (
        <Skeleton variant="text" />
      )}
    </BasicCard>
  );
}
