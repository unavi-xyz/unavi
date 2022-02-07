import { Dispatch, SetStateAction, useState } from "react";
import { IconButton, Menu, MenuItem } from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";

interface Props {
  id: string;
  setDeleted: Dispatch<SetStateAction<boolean>>;
}

export default function SceneActions({ id, setDeleted }: Props) {
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
    const str = localStorage.getItem("scenes");
    const list = JSON.parse(str);
    const newList = list.filter((item) => item !== id);
    localStorage.setItem("scenes", JSON.stringify(newList));

    localStorage.setItem(`${id}-name`, null);
    localStorage.setItem(`${id}-preview`, null);
    localStorage.setItem(`${id}-scene`, null);

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
