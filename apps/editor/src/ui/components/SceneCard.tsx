import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Typography,
} from "@mui/material";
import Link from "next/link";
import { Room } from "matrix-js-sdk";

import { useIdenticon } from "ui";

interface Props {
  room: Room;
}

export default function SceneCard({ room }: Props) {
  const identicon = useIdenticon(room.roomId);

  return (
    <Grid item xs={12} sm={6} md={4} xl={3}>
      <Card elevation={4}>
        <Link href={`/scene/${room.roomId}`} passHref>
          <CardActionArea>
            <CardMedia component="img" height="140px" image={identicon} />
            <CardContent>
              <Typography>{room.name}</Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
