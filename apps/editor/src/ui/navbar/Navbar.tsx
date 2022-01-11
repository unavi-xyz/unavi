import { useContext, useEffect, useState } from "react";
import { Button, Grid, Paper, Stack } from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import Link from "next/link";

import { ClientContext, useRoomFromId } from "matrix";
import ColorIconButton from "../components/ColorIconButton";
import SceneName from "./SceneName";
import Tools from "./Tools";

export default function Navbar() {
  const { client } = useContext(ClientContext);

  const [id, setId] = useState("");

  const room = useRoomFromId(client, id);

  useEffect(() => {
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);
    setId(urlParams.get("scene"));
  }, []);

  return (
    <Paper square variant="outlined" style={{ padding: "0.2rem" }}>
      <Grid container alignItems="center">
        <Grid item xs={4}>
          <Stack
            direction="row"
            alignItems="center"
            justifyContent="flex-start"
            spacing={1}
          >
            <Link href={`/scene/${id}`} passHref>
              <span>
                <ColorIconButton>
                  <ArrowBackIosNewIcon className="NavbarIcon" />
                </ColorIconButton>
              </span>
            </Link>

            <SceneName name={room?.name} />
          </Stack>
        </Grid>

        <Grid item xs={4}>
          <Tools />
        </Grid>

        <Grid item xs={4}>
          <Stack direction="row" justifyContent="flex-end" spacing={2}>
            <ColorIconButton tooltip="Download">
              <DownloadIcon className="NavbarIcon" />
            </ColorIconButton>

            <Button
              variant="contained"
              color="secondary"
              size="small"
              style={{
                marginTop: 5,
                marginBottom: 5,
                marginRight: 5,
                paddingLeft: 16,
                paddingRight: 16,
              }}
            >
              Publish
            </Button>
          </Stack>
        </Grid>
      </Grid>
    </Paper>
  );
}
