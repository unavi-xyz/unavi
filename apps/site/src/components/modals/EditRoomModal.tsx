import { useContext, useEffect, useState } from "react";
import { Button, Checkbox, Stack, TextField, Typography } from "@mui/material";
import { DIDDataStore } from "@glazed/did-datastore";
import { useRouter } from "next/router";
import { BasicModal } from "ui";
import { ceramic, CeramicContext, merge, useRoom, useRooms } from "ceramic";

const model = require("ceramic/models/Rooms/model.json");

interface Props {
  open: boolean;
  handleClose: () => void;
}

export default function EditRoomModal({ open, handleClose }: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const { id: userId } = useContext(CeramicContext);

  const { room } = useRoom(id);
  const rooms = useRooms(userId);

  const [name, setName] = useState("");
  const [checked, setChecked] = useState(false);

  useEffect(() => {
    if (!room) return;
    setName(room.name ?? "");
  }, [room]);

  useEffect(() => {
    if (!rooms) return;
    const pinned = rooms.includes(id);
    setChecked(pinned);
  }, [id, rooms]);

  async function handleSave() {
    if (checked) {
      //add tile to rooms did record
      const store = new DIDDataStore({ ceramic, model });
      const oldRooms = await store.get("rooms");
      const newRooms = oldRooms ? [...Object.values(oldRooms), id] : [id];
      await store.set("rooms", newRooms, { pin: true });
    } else {
      //remove tile from rooms
      const store = new DIDDataStore({ ceramic, model });
      const data = Object.values(await store.get("rooms", userId));
      const newRooms = data.filter((roomId) => roomId !== id);
      await store.set("rooms", newRooms, { pin: true });
    }

    const newContent = { name };
    await merge(id, newContent, checked);

    location.reload();
    handleClose();
  }

  return (
    <BasicModal open={open} handleClose={handleClose} title="Room Settings">
      <Stack spacing={3}>
        <Stack direction="row" alignItems="center" spacing={1}>
          <Typography>Save to profile?</Typography>
          <Checkbox
            checked={checked}
            onChange={(e) => setChecked(e.target.checked)}
          />
        </Stack>

        <TextField
          variant="standard"
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
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
