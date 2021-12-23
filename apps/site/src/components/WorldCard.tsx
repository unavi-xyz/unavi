import Link from "next/link";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Typography,
} from "@mui/material";
import { IPublicRoomsChunkRoom } from "matrix-js-sdk";

interface Props {
  world: IPublicRoomsChunkRoom;
}

export default function WorldCard({ world }: Props) {
  return (
    <Grid item xs={12} sm={6} md={4}>
      <Card elevation={4}>
        <Link href={`/home/world/${world.room_id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={world.avatar_url ?? "/imagefallback.jpg"}
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
