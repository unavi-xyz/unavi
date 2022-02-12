import { useContext } from "react";
import { Button, Grid, Paper, Stack } from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import VisibilityIcon from "@mui/icons-material/Visibility";
import { DIDDataStore } from "@glazed/did-datastore";
import { useRouter } from "next/router";
import { ColorIconButton } from "ui";
import { CeramicContext, ceramic, loader, World } from "ceramic";
import { ASSET_NAMES } from "3d";

import { useStore } from "../../state/useStore";
import { EditorScene, useScene } from "../../state/useScene";

import SceneName from "./SceneName";
import Tools from "./Tools";

const worldModel = require("ceramic/models/World/model.json");
const worldsModel = require("ceramic/models/Worlds/model.json");

const worldSchemaId = worldModel.schemas.World;

export default function Navbar() {
  const router = useRouter();

  const { authenticated } = useContext(CeramicContext);

  const id = useStore((state) => state.id);
  const scene = useScene((state) => state.scene);
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

    router.push(`/home/scene/${id}`);
  }

  async function handlePublish() {
    const name = localStorage.getItem(`${id}-name`);
    const description = localStorage.getItem(`${id}-description`);
    const image = localStorage.getItem(`${id}-preview`);

    const objects = Object.values(scene).map((obj) => obj.instance);

    const spawn = getSpawn(scene);

    const world: World = { name, description, image, spawn, scene: objects };

    //create tile
    const stream = await loader.create(
      world,
      { schema: worldSchemaId },
      { pin: true }
    );
    const streamId = stream.id.toString();

    //add tile to worlds did record
    const store = new DIDDataStore({ ceramic, model: worldsModel });
    const oldWorlds = await store.get("worlds");
    const newWorlds = oldWorlds
      ? [...Object.values(oldWorlds), streamId]
      : [streamId];
    await store.set("worlds", newWorlds, { pin: true });

    router.push(`/home/world/${streamId}`);
  }

  function handlePlay() {
    setPlayMode(true);
  }

  function handleDownload() {
    save();
    const json = toJSON();
    downloadJson(json, "scene.json");
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

              <ColorIconButton onClick={handleDownload} tooltip="Download">
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

function getSpawn(scene: EditorScene) {
  const object = Object.values(scene).find(
    (obj) => obj.instance.type === ASSET_NAMES.Spawn
  );

  if (!object) return;

  const spawn = object.instance.params.position;
  spawn[1] += 2;
  return spawn;
}

function downloadJson(text, name) {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([text], { type: "application/json" }));
  a.download = name;
  a.click();
}
