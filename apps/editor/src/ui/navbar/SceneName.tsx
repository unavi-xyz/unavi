import { useEffect, useState } from "react";
import { Button, Stack, TextField, Typography, useTheme } from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";

import BasicModal from "../components/BasicModal";

interface Props {
  id: string;
}

export default function SceneName({ id }: Props) {
  const theme = useTheme();

  const [hover, setHover] = useState(false);
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [displayedName, setDisplayedName] = useState("");

  useEffect(() => {
    const name = localStorage.getItem(`${id}-name`) ?? "";
    setName(name);
    setDisplayedName(name);

    const description = localStorage.getItem(`${id}-description`) ?? "";
    setDescription(description);
  }, [id]);

  function handleMouseOver() {
    setHover(true);
  }

  function handleMouseOut() {
    setHover(false);
  }

  function handleClick() {
    setOpen(true);
  }

  function handleClose() {
    setName(localStorage.getItem(`${id}-name`) ?? "");
    setDescription(localStorage.getItem(`${id}-description`) ?? "");
    setOpen(false);
  }

  function handleNameChange(e: any) {
    setName(e.target.value);
  }

  function handleDescriptionChange(e: any) {
    setDescription(e.target.value);
  }

  async function handleSave() {
    localStorage.setItem(`${id}-name`, name);
    localStorage.setItem(`${id}-description`, description);

    setDisplayedName(name);
    handleClose();
  }

  return (
    <div>
      <Stack
        className="clickable"
        direction="row"
        alignItems="center"
        spacing={1}
        style={{ marginLeft: 5 }}
        onClick={handleClick}
      >
        <Typography variant="h6">{displayedName}</Typography>

        <EditIcon
          className="NavbarIcon"
          onMouseOver={handleMouseOver}
          onMouseOut={handleMouseOut}
          style={{
            background: "rgba(0,0,0,0)",
            color: hover ? theme.palette.secondary.main : "rgba(0,0,0,0.2)",
          }}
        />
      </Stack>

      <BasicModal open={open} handleClose={handleClose} title="Scene Details">
        <TextField
          variant="standard"
          label="Title"
          value={name}
          onChange={handleNameChange}
        />

        <TextField
          variant="standard"
          label="Description"
          value={description}
          onChange={handleDescriptionChange}
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
      </BasicModal>
    </div>
  );
}
