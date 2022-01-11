import { useContext, useEffect, useState } from "react";
import { Grid, IconButton, Stack, Tooltip, Typography } from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";
import { useRouter } from "next/router";
import { Room } from "matrix-js-sdk";

import { ClientContext, createScene, getScenes } from "matrix";
import SceneCard from "../src/ui/components/SceneCard";
import SidebarLayout from "../src/layouts/SidebarLayout";

export default function Scenes() {
  const router = useRouter();

  const { client } = useContext(ClientContext);

  const [scenes, setScenes] = useState<Room[]>([]);

  async function handleModalOpen() {
    const room = await createScene(client, "New Scene");
    router.push(`/editor?scene=${room.room_id}`);
  }

  useEffect(() => {
    if (!client) return;

    getScenes(client).then((res) => {
      setScenes(res);
    });
  }, [client]);

  return (
    <div>
      <Grid className="page" container direction="column" rowSpacing={4}>
        <Grid item>
          <Stack direction="row" alignItems="center" spacing={1}>
            <Typography variant="h2">🌲 Scenes</Typography>

            <span>
              <Tooltip title="New Scene">
                <IconButton onClick={handleModalOpen}>
                  <AddBoxOutlinedIcon />
                </IconButton>
              </Tooltip>
            </span>
          </Stack>
        </Grid>

        <Grid item>
          {scenes.length === 0 ? (
            <div>
              <Typography>
                It looks like you don{"'"}t have any scenes.
              </Typography>
              <Typography>
                <Typography
                  className="link"
                  color="secondary"
                  component="span"
                  onClick={handleModalOpen}
                >
                  Click Here
                </Typography>{" "}
                to get started.
              </Typography>
            </div>
          ) : (
            <Grid container spacing={3}>
              {scenes.map((scene) => {
                return <SceneCard key={scene.roomId} room={scene} />;
              })}
            </Grid>
          )}
        </Grid>
      </Grid>
    </div>
  );
}

Scenes.Layout = SidebarLayout;
