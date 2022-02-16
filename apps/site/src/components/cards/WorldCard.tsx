import { useEffect, useState } from "react";
import { Skeleton, Typography } from "@mui/material";
import { useWorld } from "ceramic";

import { useIdenticon } from "../../hooks/useIdenticon";
import BasicCard from "./BasicCard";

interface Props {
  id: string;
}

export default function WorldCard({ id }: Props) {
  const { world } = useWorld(id);
  const identicon = useIdenticon(id);

  const [name, setName] = useState("");

  useEffect(() => {
    if (!world) return;
    if (world.name?.length > 0) setName(world.name);
    else setName(id);
  }, [id, world]);

  return (
    <BasicCard href={`/home/world/${id}`} image={world?.image ?? identicon}>
      {name ? (
        <Typography style={{ wordBreak: "break-word" }}>🌏 {name}</Typography>
      ) : (
        <Skeleton variant="text" />
      )}
    </BasicCard>
  );
}
