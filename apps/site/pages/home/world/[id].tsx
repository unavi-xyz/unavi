import { useContext } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import { BackNavbar, useIdenticon } from "ui";
import { CeramicContext, useScene, Room } from "ceramic";

import HomeLayout from "../../../src/layouts/HomeLayout";

import { customAlphabet } from "nanoid";
const nanoid = customAlphabet("1234567890", 8);

const roomModel = require("ceramic/models/Room/model.json");
const roomSchemaId = roomModel.schemas.Scene;

export default function World() {
  const router = useRouter();
  const id = router.query.id as string;

  const { loader } = useContext(CeramicContext);

  const world = useScene(id);
  const identicon = useIdenticon(id);

  async function handleNewRoom() {
    if (!id || !world) return;

    const name = `${world.name}#${nanoid()}`;
    const room: Room = { name, sceneStreamId: id };

    //create tile
    const stream = await loader.create(room, { schema: roomSchemaId });
    const streamId = stream.id.toString();

    const url = `/home/room/${streamId}`;
    router.push(url);
  }

  return (
    <Grid container direction="column">
      <Grid item>
        <BackNavbar text={world?.name} />
      </Grid>

      <Grid item>
        <img
          src={world?.image ?? identicon}
          alt="world image"
          style={{
            width: "100%",
            height: "400px",
            objectFit: "cover",
            borderBottom: "1px solid rgba(0,0,0,.1)",
          }}
        />
      </Grid>

      <Grid item sx={{ padding: 4 }}>
        <Stack spacing={2}>
          <Stack
            direction="row"
            justifyContent="space-between"
            alignItems="center"
          >
            <Typography variant="h4" style={{ wordBreak: "break-word" }}>
              {world?.name}
            </Typography>

            <Button
              variant="contained"
              color="secondary"
              size="large"
              onClick={handleNewRoom}
            >
              Create Room
            </Button>
          </Stack>

          {/* <Stack direction="row" alignItems="center" spacing={1}>
              <Typography variant="h6">Author:</Typography>
              <Link href={`/home/user/${world?.author?.userId}`} passHref>
                <Typography className="link" variant="h6">
                  {world?.author?.displayName}
                </Typography>
              </Link>
            </Stack> */}

          <Typography>{world?.description}</Typography>
        </Stack>
      </Grid>
    </Grid>
  );
}

World.Layout = HomeLayout;
