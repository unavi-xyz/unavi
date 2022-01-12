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

interface Props {
  room: IPublicRoomsChunkRoom;
}

export default function RoomCard({ room }: Props) {
  const identicon = useIdenticon(room.room_id);

  return (
    <Grid item xs={12} sm={6} md={4} xl={3}>
      <Card variant="outlined">
        <Link href={`/home/room/${room.room_id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={room.avatar_url ?? identicon}
            />
            <CardContent style={{ borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              <Typography>{room.name}</Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
