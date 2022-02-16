import { useContext, useEffect, useState } from "react";
import { Button, Checkbox, Stack, TextField, Typography } from "@mui/material";
import { LoadingButton } from "@mui/lab";
import { useRouter } from "next/router";
import { BasicModal } from "ui";
import {
  addRoom,
  ceramic,
  CeramicContext,
  merge,
  removeRoom,
  useRoom,
  useRooms,
  useWorld,
} from "ceramic";

interface Props {
  open: boolean;
  handleClose: () => void;
}

export default function EditRoomModal({ open, handleClose }: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const { userId } = useContext(CeramicContext);

  const { room } = useRoom(id);
  const { world } = useWorld(room?.worldStreamId);
  const rooms = useRooms(userId);

  const [name, setName] = useState("");
  const [checked, setChecked] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!room) return;
    setName(room.name ?? world?.name ?? "");
  }, [room, world?.name]);

  useEffect(() => {
    if (!rooms) return;
    const pinned = rooms.includes(id);
    setChecked(pinned);
  }, [id, rooms]);

  async function handleSave() {
    setLoading(true);
    if (checked) addRoom(id, ceramic);
    else removeRoom(id, userId, ceramic);

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
          <LoadingButton
            loading={loading}
            variant="contained"
            color="secondary"
            sx={{ width: "100%" }}
            onClick={handleSave}
          >
            Save
          </LoadingButton>
        </Stack>
      </Stack>
    </BasicModal>
  );
}
