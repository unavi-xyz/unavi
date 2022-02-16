import { useContext, useState } from "react";
import { Button, Grid, Paper, Stack } from "@mui/material";
import { LoadingButton } from "@mui/lab";
import DownloadIcon from "@mui/icons-material/Download";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import VisibilityIcon from "@mui/icons-material/Visibility";
import { useRouter } from "next/router";
import { ColorIconButton } from "ui";
import {
  CeramicContext,
  ceramic,
  loader,
  World,
  addWorld,
  newWorld,
} from "ceramic";

import { useStore } from "../../hooks/useStore";
import SceneName from "./SceneName";
import Tools from "./Tools";
import { ASSET_NAMES, PARAM_NAMES } from "3d";

export default function Navbar() {
  const router = useRouter();

  const { authenticated } = useContext(CeramicContext);

  const sceneId = useStore((state) => state.sceneId);
  const toJSON = useStore((state) => state.toJSON);
  const setPlayMode = useStore((state) => state.setPlayMode);
  const objects = useStore((state) => state.objects);

  const [loading, setLoading] = useState(false);

  async function handleBack() {
    const canvas = document.querySelector("canvas");
    const image = canvas.toDataURL("image/jpeg");
    localStorage.setItem(`${sceneId}-preview`, image);

    router.push(`/home/scene/${sceneId}`);
  }

  async function handlePublish() {
    setLoading(true);

    const name = localStorage.getItem(`${sceneId}-name`);
    const description = localStorage.getItem(`${sceneId}-description`);
    const image = localStorage.getItem(`${sceneId}-preview`);

    const world: World = {
      name,
      description,
      image,
      spawn: [0, 0, 0],
      objects: {},
    };

    Object.values(objects).forEach((obj) => {
      world.objects[obj.instance.id] = obj.instance;

      if (obj.instance.type === ASSET_NAMES.Spawn) {
        world.spawn = obj.instance.params[PARAM_NAMES.position];
      }
    });

    //upload to ceramic
    const streamId = await newWorld(world, loader);
    await addWorld(streamId, ceramic);

    router.push(`/home/world/${streamId}`);
  }

  function handlePlay() {
    setPlayMode(true);
  }

  function handleDownload() {
    const name = localStorage.getItem(`${sceneId}-name`);
    const fileName = name?.length > 0 ? name : "scene";
    const json = toJSON();
    downloadJson(json, `${fileName}.json`);
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

            <SceneName id={sceneId} />
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

            <LoadingButton
              loading={loading}
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
            </LoadingButton>
          </Stack>
        </Grid>
      </Grid>
    </Paper>
  );
}

function downloadJson(text, name) {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([text], { type: "application/json" }));
  a.download = name;
  a.click();
}
