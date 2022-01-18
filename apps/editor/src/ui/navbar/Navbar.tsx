import { useContext } from "react";
import { Button, Grid, Paper, Stack } from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import { useRouter } from "next/router";
import { ClientContext, createWorld, useRoom } from "matrix";
import { ColorIconButton, getHomeUrl } from "ui";

import { useStore } from "../../state/useStore";
import { useScene } from "../../state/useScene";

import SceneName from "./SceneName";
import Tools from "./Tools";

export default function Navbar() {
  const router = useRouter();

  const { client } = useContext(ClientContext);

  const roomId = useStore((set) => set.roomId);
  const save = useScene((set) => set.save);
  const toJSON = useScene((set) => set.toJSON);

  const room = useRoom(client, roomId);

  async function handleBack() {
    const canvas = document.querySelector("canvas");
    const image = canvas.toDataURL("image/jpeg");
    localStorage.setItem(`${roomId}-preview`, image);

    save();
    const json = toJSON();
    const time = Date.now();
    await client.sendStateEvent(roomId, "wired.scene", { json, time }, "wired");

    router.push(`/scene/${roomId}`);
  }

  async function handlePublish() {
    if (!room) return;

    save();
    const json = toJSON();

    const world = await createWorld(client, room.name, json);

    const url = `${getHomeUrl()}/world/${world.room_id}`;
    router.push(url);
  }

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
            <ColorIconButton onClick={handleBack}>
              <ArrowBackIosNewIcon className="NavbarIcon" />
            </ColorIconButton>

            <SceneName room={room} />
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
              onClick={handlePublish}
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
