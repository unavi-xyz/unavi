import { useEffect, useState } from "react";
import {
  Box,
  Grid,
  IconButton,
  Stack,
  Tooltip,
  Typography,
} from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";
import FileUploadIcon from "@mui/icons-material/FileUpload";
import { useRouter } from "next/router";
import { customAlphabet } from "nanoid";
import { BackNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";
import SceneCard from "../../src/components/SceneCard";

const nanoid = customAlphabet("1234567890", 16);

export default function Scenes() {
  const router = useRouter();

  const [scenes, setScenes] = useState([]);
  const [uploaded, setUploaded] = useState<File>();

  useEffect(() => {
    const str = localStorage.getItem("scenes") ?? "[]";
    const list = JSON.parse(str);
    setScenes(list);
  }, []);

  useEffect(() => {
    if (!uploaded) return;

    const reader = new FileReader();
    reader.onload = (e) => handleNewScene(e.target.result as string);
    reader.readAsText(uploaded);

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [uploaded]);

  async function handleNewScene(startingScene = "[]") {
    const id = nanoid();

    const str = localStorage.getItem("scenes");
    const list = JSON.parse(str) ?? [];

    list.push(id);

    localStorage.setItem("scenes", JSON.stringify(list));
    localStorage.setItem(`${id}-name`, "New Scene");
    localStorage.setItem(`${id}-scene`, startingScene);

    router.push(`/home/scene/${id}`);
  }

  function onDelete(id: string) {
    const newScenes = scenes.filter((scene) => scene !== id);
    setScenes(newScenes);
  }

  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text="Editor" />
      </Grid>

      <Stack
        direction="row"
        alignItems="center"
        justifyContent="space-between"
        sx={{ padding: 4, paddingBottom: 0 }}
      >
        <Typography variant="h4">Scenes</Typography>

        <Stack direction="row" spacing={1}>
          <div>
            <input
              accept="application/json"
              style={{ display: "none" }}
              id="scene-import-input"
              multiple
              type="file"
              onChange={(e) => setUploaded(e.target.files[0])}
            />

            <label htmlFor="scene-import-input">
              <Tooltip title="Import Scene">
                <IconButton component="span">
                  <FileUploadIcon />
                </IconButton>
              </Tooltip>
            </label>
          </div>

          <span>
            <Tooltip title="New Scene">
              <IconButton onClick={() => handleNewScene()}>
                <AddBoxOutlinedIcon />
              </IconButton>
            </Tooltip>
          </span>
        </Stack>
      </Stack>

      <Box sx={{ padding: 4 }}>
        {!scenes || scenes.length === 0 ? (
          <div>
            <Typography>
              It looks like you don{"'"}t have any scenes.
            </Typography>
            <Typography>
              <Typography
                className="link"
                color="secondary"
                component="span"
                onClick={() => handleNewScene()}
              >
                Click Here
              </Typography>{" "}
              to get started.
            </Typography>
          </div>
        ) : (
          <Grid container spacing={3}>
            {scenes.map((id) => {
              return <SceneCard key={id} id={id} onDelete={onDelete} />;
            })}
          </Grid>
        )}
      </Box>
    </Grid>
  );
}

Scenes.Layout = HomeLayout;
