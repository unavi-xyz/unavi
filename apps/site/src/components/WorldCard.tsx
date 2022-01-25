import { useContext } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Typography,
} from "@mui/material";
import Link from "next/link";
import { IPublicRoomsChunkRoom } from "matrix-js-sdk";
import { useIdenticon } from "ui";
import { ClientContext, useRoomAvatar } from "matrix";

interface Props {
  world: IPublicRoomsChunkRoom;
}

export default function WorldCard({ world }: Props) {
  const { client } = useContext(ClientContext);

  const identicon = useIdenticon(world.room_id);
  const avatar = useRoomAvatar(client, world);

  return (
    <Grid item xs={12} sm={6} md={4} xl={3}>
      <Card variant="outlined">
        <Link href={`/home/world/${world.room_id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={avatar ?? identicon}
            />
            <CardContent style={{ borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              <Typography>{world.name}</Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
