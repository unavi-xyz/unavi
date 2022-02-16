import {
  Card,
  CardActionArea,
  CircularProgress,
  Grid,
  Stack,
  Tooltip,
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import { useRouter } from "next/router";
import { loader, newRoom, Room } from "ceramic";
import { useState } from "react";

interface Props {
  worldId: string;
}

export default function NewRoomCard({ worldId }: Props) {
  const router = useRouter();

  const [loading, setLoading] = useState(false);

  async function handleClick() {
    setLoading(true);

    const room: Room = { worldStreamId: worldId };
    const streamId = await newRoom(room, loader);

    const url = `/home/room/${streamId}`;
    router.push(url);
  }

  return (
    <Grid item xs={12} sm={6} md={4}>
      <Tooltip title={loading ? "" : "New Room"}>
        <Card variant="outlined" sx={{ borderRadius: 0, height: "199px" }}>
          <CardActionArea
            disabled={loading}
            onClick={handleClick}
            sx={{ height: "100%" }}
          >
            <Stack alignItems="center" justifyContent="center">
              <Stack
                direction="row"
                alignItems="center"
                justifyContent="center"
              >
                {loading ? (
                  <CircularProgress color="info" />
                ) : (
                  <AddIcon fontSize="large" sx={{ color: "GrayText" }} />
                )}
              </Stack>
            </Stack>
          </CardActionArea>
        </Card>
      </Tooltip>
    </Grid>
  );
}
