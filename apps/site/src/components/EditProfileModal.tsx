import { useContext, useEffect, useState } from "react";
import { Button, Stack, TextField } from "@mui/material";
import { BasicModal } from "ui";
import { CeramicContext, useProfile } from "ceramic";

interface Props {
  open: boolean;
  handleClose: () => void;
}

export default function EditProfileModal({ open, handleClose }: Props) {
  const { id } = useContext(CeramicContext);

  const { profile, merge } = useProfile(id);

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  useEffect(() => {
    setName(profile?.name);
    setDescription(profile?.description);
  }, [profile]);

  async function handleSave() {
    await merge({ name, description });
    location.reload();
    handleClose();
  }

  return (
    <BasicModal open={open} handleClose={handleClose} title="Edit Profile">
      <Stack spacing={3}>
        <TextField
          variant="standard"
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />

        <TextField
          variant="standard"
          label="Bio"
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
