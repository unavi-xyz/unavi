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
  room: IPublicRoomsChunkRoom;
}

export default function RoomCard({ room }: Props) {
  return (
    <Grid item xs={12} sm={6} md={4}>
      <Card elevation={4}>
        <Link href={`/home/room/${room.room_id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={room.avatar_url ?? "/imagefallback.jpg"}
            />
            <CardContent>
              <Typography>{room.name}</Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
