import { useEffect, useState } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Stack,
  Typography,
} from "@mui/material";
import Link from "next/link";
import { Room } from "matrix-js-sdk";
import { useIdenticon } from "ui";

import SceneActions from "./SceneActions";

interface Props {
  room: Room;
}

export default function SceneCard({ room }: Props) {
  const identicon = useIdenticon(room.roomId);

  const [hover, setHover] = useState(false);
  const [deleted, setDeleted] = useState(false);
  const [preview, setPreview] = useState("");

  useEffect(() => {
    setPreview(localStorage.getItem(`${room?.roomId}-preview`));
  }, [room?.roomId]);

  if (deleted) return <></>;

  return (
    <Grid item xs={12} sm={6} md={4} xl={3}>
      <Card variant="outlined">
        <Link href={`/scene/${room.roomId}`} passHref>
          <CardActionArea
            onMouseOver={() => setHover(true)}
            onMouseOut={() => setHover(false)}
          >
            <CardMedia
              component="img"
              height="140px"
              image={preview ?? identicon}
            />
            <CardContent sx={{ p: 1, borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              <Stack
                direction="row"
                justifyContent="space-between"
                alignItems="center"
              >
                <Typography>{room.name}</Typography>
                <span style={{ visibility: hover ? "visible" : "hidden" }}>
                  <SceneActions roomId={room.roomId} setDeleted={setDeleted} />
                </span>
              </Stack>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
