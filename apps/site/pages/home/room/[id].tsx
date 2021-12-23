import React, { useContext, useEffect, useState } from "react";
import { useRouter } from "next/router";
import { Button, Grid, Typography } from "@mui/material";

import HomeLayout from "../../../src/layouts/HomeLayout";
import { MatrixContext } from "../../../src/matrix/MatrixProvider";
import { getRoom } from "../../../src/matrix/rooms";

function getSubdomain(hostname) {
  var regexParse = new RegExp("[a-z-0-9]{2,63}.[a-z.]{2,5}$");
  var urlParts = regexParse.exec(hostname);
  return hostname.replace(urlParts[0], "").slice(0, -1);
}

export default function Id() {
  const router = useRouter();
  const id = `${router.query.id}`;

  const { client } = useContext(MatrixContext);

  const [roomURL, setRoomURL] = useState("");

  useEffect(() => {
    const subdomain = getSubdomain(window.location.hostname);

    //in production, direct the user to app.domain.com
    //in development, direct them to the port the app is on
    if (subdomain === "www") {
      setRoomURL(
        `https://${window.location.hostname.replace("www", "app")}?room=${id}`
      );
    } else {
      setRoomURL(`http://localhost:3000?room=${id}`);
    }
  }, [id]);

  useEffect(() => {
    if (!client) return;
    getRoom(client, id).then((res) => {
      console.log(res);
    });
  }, [client, id]);

  return (
    <Grid
      className="container underNavbar"
      container
      direction="column"
      rowSpacing={4}
    >
      <Grid item>
        <Typography variant="h4" style={{ wordBreak: "break-word" }}>
          🚪 {id}
        </Typography>
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
