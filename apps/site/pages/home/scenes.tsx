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
import { useRouter } from "next/router";
import { customAlphabet } from "nanoid";
import { BackNavbar } from "ui";

import HomeLayout from "../../src/layouts/HomeLayout";
import SceneCard from "../../src/components/SceneCard";

const nanoid = customAlphabet("1234567890", 16);

export default function Scenes() {
  const router = useRouter();

  const [scenes, setScenes] = useState([]);

  async function handleNewScene() {
    const id = nanoid();

    const str = localStorage.getItem("scenes");
    const list = JSON.parse(str) ?? [];

    list.push(id);

    localStorage.setItem("scenes", JSON.stringify(list));
    localStorage.setItem(`${id}-name`, "New Scene");
    localStorage.setItem(`${id}-scene`, JSON.stringify([]));

    router.push(`/home/scene/${id}`);
  }

  useEffect(() => {
    const str = localStorage.getItem("scenes") ?? "[]";
    const list = JSON.parse(str);
    setScenes(list);
  }, []);

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
        spacing={1}
        sx={{ padding: 4, paddingBottom: 0 }}
      >
        <Typography variant="h4">Scenes</Typography>

        <span>
          <Tooltip title="New Scene">
            <IconButton onClick={handleNewScene}>
              <AddBoxOutlinedIcon />
            </IconButton>
          </Tooltip>
        </span>
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
                onClick={handleNewScene}
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
