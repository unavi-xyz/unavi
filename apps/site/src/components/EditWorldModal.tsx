import { useContext, useEffect, useState } from "react";
import { Button, Stack, TextField } from "@mui/material";
import { DIDDataStore } from "@glazed/did-datastore";
import { useRouter } from "next/router";
import { BasicModal } from "ui";
import { ceramic, CeramicContext, merge, unpin, useWorld } from "ceramic";

const model = require("ceramic/models/Worlds/model.json");

interface Props {
  open: boolean;
  handleClose: () => void;
}

export default function EditWorldModal({ open, handleClose }: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const { id: userId } = useContext(CeramicContext);

  const { world } = useWorld(id);

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  useEffect(() => {
    if (!world) return;

    setName(world.name ?? "");
    setDescription(world.description ?? "");
  }, [world]);

  async function handleSave() {
    const newContent = { name, description };
    await merge(id, newContent);

    location.reload();
    handleClose();
  }

  async function handleDelete() {
    //unpin the tile
    await unpin(id);

    //remove from user worlds
    const store = new DIDDataStore({ ceramic, model });
    const data = (await store.get("worlds", userId)) as string[];
    const newWorlds = data.filter((worldId) => worldId !== id);
    await store.set("worlds", newWorlds, { pin: true });

    handleClose();
  }

  return (
    <BasicModal open={open} handleClose={handleClose} title="World Settings">
      <Stack spacing={3}>
        <Button
          variant="contained"
          color="primary"
          sx={{ width: "100%" }}
          onClick={handleDelete}
        >
          Delete World
        </Button>

        <TextField
          variant="standard"
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />

        <TextField
          variant="standard"
          label="Description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />

        <Stack direction="row" spacing={1}>
          <Button
            variant="contained"
            color="info"
            sx={{ width: "100%" }}
            onClick={handleClose}
          >
            Cancel
          </Button>
          <Button
            variant="contained"
            color="secondary"
            sx={{ width: "100%" }}
            onClick={handleSave}
          >
            Save
          </Button>
        </Stack>
      </Stack>
    </BasicModal>
  );
}
