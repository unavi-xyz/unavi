import { useContext, useEffect, useState } from "react";
import { Button, Stack, TextField, Typography, useTheme } from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";
import { Room } from "matrix-js-sdk";

import { ClientContext } from "matrix";
import BasicModal from "../components/BasicModal";

interface Props {
  room: null | Room;
}

export default function SceneName({ room }: Props) {
  const theme = useTheme();

  const { client } = useContext(ClientContext);

  const [hover, setHover] = useState(false);
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState(room?.name);
  const [description, setDescription] = useState("");
  const [displayedName, setDisplayedName] = useState("");

  useEffect(() => {
    setTitle(room?.name);
    setDisplayedName(room?.name);
  }, [room]);

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
    setOpen(false);
  }

  function handleTitleChange(e: any) {
    setTitle(e.target.value);
  }

  function handleDescriptionChange(e: any) {
    setDescription(e.target.value);
  }

  async function handleSave() {
    await client.setRoomName(room?.roomId, title);
    setDisplayedName(title);
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
          value={title}
          onChange={handleTitleChange}
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
