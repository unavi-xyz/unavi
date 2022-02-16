import { useEffect, useState } from "react";
import { Button, Stack, TextField } from "@mui/material";
import { useRouter } from "next/router";

import BasicModal from "./BasicModal";

interface Props {
  id: string;
  open: boolean;
  handleClose: () => void;
}

export default function EditSceneModal({ id, open, handleClose }: Props) {
  const router = useRouter();

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  useEffect(() => {
    setName(localStorage.getItem(`${id}-name`) ?? "");
    setDescription(localStorage.getItem(`${id}-description`) ?? "");
  }, [id]);

  async function handleSave() {
    localStorage.setItem(`${id}-name`, name ?? "");
    localStorage.setItem(`${id}-description`, description ?? "");

    location.reload();
    handleClose();
  }

  async function handleDelete() {
    const str = localStorage.getItem("scenes");
    const list = JSON.parse(str);
    const newList = list.filter((item) => item !== id);
    localStorage.setItem("scenes", JSON.stringify(newList));

    localStorage.setItem(`${id}-name`, null);
    localStorage.setItem(`${id}-description`, null);
    localStorage.setItem(`${id}-preview`, null);
    localStorage.setItem(`${id}-scene`, null);

    handleClose();
    router.push("/home/scenes");
  }

  return (
    <BasicModal open={open} handleClose={handleClose} title="Scene Settings">
      <Stack spacing={3}>
        <Button
          variant="contained"
          color="primary"
          sx={{ width: "100%" }}
          onClick={handleDelete}
        >
          Delete Scene
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
