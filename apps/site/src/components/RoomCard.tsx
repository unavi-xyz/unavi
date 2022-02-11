import { useEffect, useState } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Typography,
} from "@mui/material";
import Link from "next/link";
import { useIdenticon } from "ui";
import { useRoom, useWorld } from "ceramic";

interface Props {
  id: string;
}

export default function RoomCard({ id }: Props) {
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

  if (!room) return <></>;

  return (
    <Grid item xs={12} sm={6} md={4}>
      <Card variant="outlined">
        <Link href={`/home/room/${id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={world?.image ?? identicon}
            />
            <CardContent style={{ borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              <Typography style={{ wordBreak: "break-word" }}>
                {name}
              </Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
