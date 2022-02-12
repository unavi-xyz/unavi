import { useEffect, useState } from "react";
import { Stack, Typography, useTheme } from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";
import EditSceneModal from "../../../components/modals/EditSceneModal";

interface Props {
  id: string;
}

export default function SceneName({ id }: Props) {
  const theme = useTheme();

  const [hover, setHover] = useState(false);
  const [open, setOpen] = useState(false);
  const [displayedName, setDisplayedName] = useState("");

  useEffect(() => {
    const name = localStorage.getItem(`${id}-name`);
    setDisplayedName(name?.length > 0 ? name : id);
  }, [id]);

  function handleMouseOver() {
    setHover(true);
  }

  function handleMouseOut() {
    setHover(false);
  }

  return (
    <div>
      <EditSceneModal id={id} open={open} handleClose={() => setOpen(false)} />

      <Stack
        className="clickable"
        direction="row"
        alignItems="center"
        spacing={1}
        style={{ marginLeft: 5 }}
        onClick={() => setOpen(true)}
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
    </div>
  );
}
