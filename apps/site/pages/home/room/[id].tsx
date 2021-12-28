import { useContext, useEffect, useState } from "react";
import { Button, Grid, Typography } from "@mui/material";
import Link from "next/link";
import { useRouter } from "next/router";
import { IPublicRoomsChunkRoom } from "matrix-js-sdk";

import HomeLayout from "../../../src/layouts/HomeLayout";
import { MatrixContext } from "../../../src/matrix/MatrixProvider";
import { getRoom } from "../../../src/matrix/rooms";
import { parseRoomTopic } from "../../../src/matrix/topics";

function getSubdomain(hostname) {
  var regexParse = new RegExp("[a-z-0-9]{2,63}.[a-z.]{2,5}$");
  var urlParts = regexParse.exec(hostname);
  return hostname.replace(urlParts[0], "").slice(0, -1);
}

export default function Id() {
  const router = useRouter();
  const { id } = router.query;

  const { client } = useContext(MatrixContext);

  const [roomURL, setRoomURL] = useState("");
  const [room, setRoom] = useState<null | IPublicRoomsChunkRoom>(null);
  const [world, setWorld] = useState<null | IPublicRoomsChunkRoom>(null);

  useEffect(() => {
    const subdomain = getSubdomain(window.location.hostname);

    //in production, direct the user to app.domain.com
    //in development, direct them to localhost
    if (subdomain === "www") {
      setRoomURL(
        `https://${window.location.hostname.replace("www", "app")}?room=${id}`
      );
    } else {
      setRoomURL(`http://localhost:3000?room=${id}`);
    }
  }, [id]);

  useEffect(() => {
    if (!client || !id) return;
    getRoom(client, `${id}`).then((res) => {
      setRoom(res);
    });
  }, [client, id]);

  useEffect(() => {
    if (!client || !room) return;

    const worldId = parseRoomTopic(room.topic);

    getRoom(client, worldId, true).then((res) => {
      setWorld(res);
    });
  }, [client, room]);

  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
      <Grid item>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          🚪 {room?.name ?? id}
        </Typography>
      </Grid>

      <Grid item container columnSpacing={1}>
        <Grid item>
          <Typography variant="h6">World:</Typography>
        </Grid>
        <Grid item>
          <Link href={`/home/world/${world?.room_id}`} passHref>
            <Typography
              className="link"
              variant="h6"
              style={{ wordBreak: "break-word" }}
            >
              {world?.name}
            </Typography>
          </Link>
        </Grid>
      </Grid>

      <Grid item>
        <Button
          variant="contained"
          color="secondary"
          href={roomURL}
          target="_blank"
        >
          Join Room
        </Button>
      </Grid>
    </Grid>
  );
}

Id.Layout = HomeLayout;
