import { useContext } from "react";
import { Button, Grid, Paper, Stack } from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import VisibilityIcon from "@mui/icons-material/Visibility";
import { DIDDataStore } from "@glazed/did-datastore";
import { useRouter } from "next/router";
import { ColorIconButton, getHomeUrl } from "ui";
import { CeramicContext, Scene } from "ceramic";

import { useStore } from "../../state/useStore";
import { useScene } from "../../state/useScene";

import SceneName from "./SceneName";
import Tools from "./Tools";

const sceneModel = require("ceramic/models/Scene/model.json");
const worldsModel = require("ceramic/models/Worlds/model.json");

const sceneSchemaId = sceneModel.schemas.Scene;

export default function Navbar() {
  const router = useRouter();

  const { ceramic, loader, authenticated } = useContext(CeramicContext);

  const id = useStore((state) => state.id);
  const setPlayMode = useStore((state) => state.setPlayMode);
  const save = useScene((state) => state.save);
  const toJSON = useScene((state) => state.toJSON);

  async function handleBack() {
    const canvas = document.querySelector("canvas");
    const image = canvas.toDataURL("image/jpeg");
    localStorage.setItem(`${id}-preview`, image);

    save();
    const json = toJSON();
    localStorage.setItem(`${id}-scene`, json);

    router.push(`/scene/${id}`);
  }

  async function handlePublish() {
    const name = localStorage.getItem(`${id}-name`);
    const description = localStorage.getItem(`${id}-description`);
    const image = localStorage.getItem(`${id}-preview`);
    save();
    const scene = toJSON();
    const world: Scene = { name, description, image, scene };

    //create tile
    const stream = await loader.create(world, { schema: sceneSchemaId });
    const streamId = stream.id.toString();

    //add tile to worlds did record
    const store = new DIDDataStore({ ceramic, model: worldsModel });
    const oldWorlds = await store.get("worlds");
    const newWorlds = [...oldWorlds, streamId];
    await store.merge("worlds", newWorlds);

    const url = `${getHomeUrl()}/world/${streamId}`;
    router.push(url);
  }

  function handlePlay() {
    setPlayMode(true);
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

            <SceneName id={id} />
          </Stack>
        </Grid>

        <Grid item xs={4}>
          <Tools />
        </Grid>

        <Grid item xs={4}>
          <Stack direction="row" justifyContent="flex-end" spacing={2}>
            <Stack direction="row" justifyContent="flex-end" spacing={1}>
              <ColorIconButton onClick={handlePlay} tooltip="Preview">
                <VisibilityIcon className="NavbarIcon" />
              </ColorIconButton>

              <ColorIconButton tooltip="Download">
                <DownloadIcon className="NavbarIcon" />
              </ColorIconButton>
            </Stack>

            <Button
              variant="contained"
              color="secondary"
              size="small"
              onClick={handlePublish}
              disabled={!authenticated}
              sx={{
                paddingLeft: 2,
                paddingRight: 2,
              }}
              style={{
                marginTop: "4px",
                marginBottom: "4px",
                marginRight: "2px",
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
