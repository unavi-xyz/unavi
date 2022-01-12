import { Dispatch, SetStateAction, useContext, useState } from "react";
import { IconButton, Menu, MenuItem } from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import { ClientContext, deleteScene } from "matrix";

interface Props {
  roomId: string;
  setDeleted: Dispatch<SetStateAction<boolean>>;
}

export default function SceneActions({ roomId, setDeleted }: Props) {
  const { client } = useContext(ClientContext);

  const [anchorEl, setAnchorEl] = useState(null);
  const open = Boolean(anchorEl);

  function handleClick(e: any) {
    e.preventDefault();
    setAnchorEl(e.currentTarget);
  }

  function handleClose(e: any) {
    e.preventDefault();
    setAnchorEl(null);
  }

  function handleDelete(e: any) {
    handleClose(e);
    deleteScene(client, roomId);
    setDeleted(true);
  }

  return (
    <div>
      <IconButton size="small" onClick={handleClick}>
        <MoreHorizIcon />
      </IconButton>

      <Menu anchorEl={anchorEl} open={open} onClose={handleClose}>
        <MenuItem onClick={handleClose}>Duplicate</MenuItem>
        <MenuItem onClick={handleClose}>Download</MenuItem>
        <MenuItem onClick={handleDelete}>Delete</MenuItem>
      </Menu>
    </div>
  );
}
