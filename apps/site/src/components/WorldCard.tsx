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

import { useIdenticon } from "../hooks";

interface Props {
  world: IPublicRoomsChunkRoom;
}

export default function WorldCard({ world }: Props) {
  const identicon = useIdenticon(world.room_id);

  return (
    <Grid item xs={12} sm={6} md={4} xl={3}>
      <Card elevation={4}>
        <Link href={`/home/world/${world.room_id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={world.avatar_url ?? identicon}
            />
            <CardContent>
              <Typography>{world.name}</Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
