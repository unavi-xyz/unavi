import { useContext } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { customAlphabet } from "nanoid";
import { BackNavbar, useIdenticon } from "ui";
import { CeramicContext, Room, loader, useScene, useProfile } from "ceramic";

import HomeLayout from "../../../src/layouts/HomeLayout";

const nanoid = customAlphabet("1234567890", 8);

const roomModel = require("ceramic/models/Room/model.json");
const roomSchemaId = roomModel.schemas.Scene;

export default function World() {
  const router = useRouter();
  const id = router.query.id as string;

  const { authenticated } = useContext(CeramicContext);

  const identicon = useIdenticon(id);
  const { scene, author } = useScene(id);
  const { profile } = useProfile(author);

  async function handleNewRoom() {
    const name = `${scene.name}#${nanoid()}`;
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
        <BackNavbar text={scene?.name} />
      </Grid>

      <Grid item>
        <img
          src={scene?.image ?? identicon}
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
              {scene?.name}
            </Typography>

            <Button
              variant="contained"
              color="secondary"
              size="large"
              onClick={handleNewRoom}
              disabled={!authenticated}
            >
              Create Room
            </Button>
          </Stack>

          <Stack direction="row" alignItems="center" spacing={1}>
            {author && (
              <>
                <Typography variant="h6">By</Typography>
                <Link href={`/home/user/${author}`} passHref>
                  <Typography className="link" variant="h6">
                    {profile?.name ?? author}
                  </Typography>
                </Link>
              </>
            )}
          </Stack>

          <Typography>{scene?.description}</Typography>
        </Stack>
      </Grid>
    </Grid>
  );
}

World.Layout = HomeLayout;
