import { useContext, useEffect, useState } from "react";
import { Button, Grid, Stack, Typography } from "@mui/material";
import { useRouter } from "next/router";
import Link from "next/link";
import { BackNavbar, useIdenticon } from "ui";
import { CeramicContext, Room, loader, useWorld, useProfile } from "ceramic";

import HomeLayout from "../../../src/layouts/HomeLayout";
import EditWorldModal from "../../../src/components/modals/EditWorldModal";

const roomModel = require("ceramic/models/Room/model.json");
const roomSchemaId = roomModel.schemas.Scene;

export default function World() {
  const router = useRouter();
  const id = router.query.id as string;

  const { id: userId, authenticated } = useContext(CeramicContext);

  const identicon = useIdenticon(id);
  const { world, controller } = useWorld(id);
  const { profile } = useProfile(controller);

  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");

  useEffect(() => {
    if (!world) return;
    if (world?.name?.length > 0) setName(world.name);
    else setName(id);
  }, [id, world]);

  async function handleNewRoom() {
    const room: Room = { worldStreamId: id };

    //create tile
    const stream = await loader.create(room, { schema: roomSchemaId });
    const streamId = stream.id.toString();

    const url = `/home/room/${streamId}`;
    router.push(url);
  }

  return (
    <Grid container direction="column">
      <EditWorldModal open={open} handleClose={() => setOpen(false)} />

      <Grid item>
        <BackNavbar
          text={`🌏 ${name}`}
          back
          more={userId === controller ? () => setOpen(true) : undefined}
        />
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
              {name}
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
            {controller && (
              <>
                <Typography variant="h6">By</Typography>
                <Link href={`/home/user/${controller}/worlds`} passHref>
                  <Typography className="link" variant="h6">
                    {profile?.name ?? controller}
                  </Typography>
                </Link>
              </>
            )}
          </Stack>

          <Typography>{world?.description}</Typography>
        </Stack>
      </Grid>
    </Grid>
  );
}

World.Layout = HomeLayout;
